//! Proc-macro to append mappings on items for remapping.

use std::{
    collections::{HashMap, HashSet},
    fmt::Write as _,
    iter::Peekable,
};

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use smol_str::ToSmolStr as _;

/// Expected count of mapping names for one item by default.
///
/// This isn't any restriction but only a hint for allocation.
const HINT_NAMES_COUNT: usize = 2;

fn rewrite_ident_in_rust(literal: Literal) -> Ident {
    let input = literal.to_smolstr();
    // reserve more capacity for de-camelizing and reduce allocations
    let mut output = String::with_capacity(input.len() + 4);
    assert!(
        input.starts_with('"') && input[1..].ends_with('"'),
        "expected string literal for names"
    );
    let quoted = &input[1..input.len() - 1];
    let mut pascal = false;
    rustc_literal_escaper::unescape_str(quoted, |range, res| match res {
        Ok(c) => {
            if c.is_ascii_uppercase() && !pascal {
                if range.start == 0 {
                    pascal = true;
                    output.push(c);
                } else {
                    output.push('_');
                    output.push(c.to_ascii_lowercase());
                }
            } else {
                output.push(c);
            }
        }
        Err(err) => {
            if err.is_fatal() {
                panic!("failed to parse string literal");
            }
        }
    });

    // strip get_ prefixes
    let output_slice = if let Some(body) = output.strip_prefix("get_")
        && !body.starts_with("mut")
        && !body.starts_with("ref")
    {
        body
    } else {
        &output
    };

    Ident::new(output_slice, literal.span())
}

fn parse_attr(attr: TokenStream) -> HashMap<Ident, Vec<Ident>> {
    let mut attr = attr.into_iter();
    let mut names: HashMap<Ident, Vec<Ident>> = HashMap::with_capacity(HINT_NAMES_COUNT);
    let mut keys = HashSet::with_capacity(HINT_NAMES_COUNT);
    // panics are allowed inside this block as only syntax errors in attribute are reported
    while let Some(t) = attr.next() {
        let TokenTree::Ident(key) = t else {
            panic!("expected ident, found {t}")
        };

        // parse '='
        {
            let t = attr
                .next()
                .expect("expected '=' followed after mappings key");
            let TokenTree::Punct(key) = t else {
                panic!("expected '=', found {t}")
            };
            assert!(
                key.as_char() == '=' && key.spacing() == Spacing::Alone,
                "expected standalone '=', found {key}",
            );
        }

        let Some(TokenTree::Literal(name)) = attr.next() else {
            panic!("expected literal after '='")
        };
        assert!(keys.insert(key.clone()), "duplicated key \"{key}\"");
        names
            .entry(rewrite_ident_in_rust(name))
            .and_modify(|set| set.push(key.clone()))
            .or_insert_with(|| [key].into_iter().collect());

        let Some(t) = attr.next() else {
            break;
        };
        let TokenTree::Punct(punct) = t else {
            panic!("expected punctuation after mapping pair, found {t}")
        };
        assert!(
            punct.as_char() == ',' && punct.spacing() == Spacing::Alone,
            "expected standalone ',' after mapping pair, found {punct}",
        )
    }
    assert!(!names.is_empty(), "no mappings provided");
    names
}

/// Remaps a standalone function.
///
/// This couldn't be used to remap an associated function of a type or trait.
/// If so, use [`remap_method`] instead.
#[proc_macro_attribute]
#[deprecated = "discouraged to use, use `remap` instead"]
pub fn remap_fn(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    remap_fn_inner(attr.into(), item.clone().into(), false, true).map_or(item, Into::into)
}

/// Remaps an type or trait associated function, so-called a method.
#[proc_macro_attribute]
pub fn remap_method(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    remap_fn_inner(attr.into(), item.clone().into(), true, true).map_or(item, Into::into)
}

/// Remaps an item.
///
/// # Examples
///
/// ```rust
/// # use rimecraft_remap::remap;
/// #[remap(mojmaps = "ResourceLocation")]
/// pub struct Identifier;
///
/// #[remap(mojmaps = "EMPTY")]
/// pub const EMPTY: Identifier = Identifier;
/// ```
#[proc_macro_attribute]
pub fn remap(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    remap_inner(attr.into(), item.clone().into(), true).map_or(item, Into::into)
}

fn remap_inner(attr: TokenStream, item: TokenStream, cfg: bool) -> Option<TokenStream> {
    let names = parse_attr(attr);
    let mut iter = item.clone().into_iter().peekable();
    let mut vis = None;
    while let Some(t) = iter.next() {
        if let TokenTree::Ident(t) = t {
            if &t == "pub" {
                let peek = iter.peek();
                vis = Some((
                    t.clone(),
                    if let Some(TokenTree::Group(g)) = peek
                        && g.delimiter() == Delimiter::Parenthesis
                    {
                        Some(g.clone())
                    } else {
                        None
                    },
                ));
            }
            let t = t.to_smolstr();
            if matches!(
                &*t,
                "mod" | "fn" | "type" | "struct" | "enum" | "union" | "const" | "trait" | "static"
            ) {
                break;
            } else if matches!(&*t, "impl" | "extern" | "use") {
                panic!("unexpected item type \"{}\"", t)
            }
        }
    }
    let TokenTree::Ident(native_name) = iter.next()? else {
        return None;
    };

    let mut item: TokenStream = AttachDocIter {
        iter: item.into_iter().peekable(),
        done: false,
        maps: &names,
        native_name: &native_name,
        doc_buf: None,
    }
    .collect();

    for (name, keys) in names {
        if name == native_name {
            continue;
        }

        if cfg {
            item.extend(tt_cfg(name.span(), keys));
        }
        item.extend(tt_doc_hidden(name.span()));
        item.extend(tt_allow_lint("unused_imports", name.span()));
        if let Some((vis, vis_g)) = &vis {
            item.extend([TokenTree::Ident(vis.clone())]);
            if let Some(vis_g) = vis_g {
                item.extend([TokenTree::Group(vis_g.clone())]);
            }
        }
        item.extend([
            TokenTree::Ident(Ident::new("use", native_name.span())),
            TokenTree::Ident(native_name.clone()),
            TokenTree::Ident(Ident::new("as", name.span())),
            TokenTree::Ident(name),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);
    }

    Some(item)
}

fn remap_fn_inner(
    attr: TokenStream,
    item: TokenStream,
    use_self: bool,
    cfg: bool,
) -> Option<TokenStream> {
    let names = parse_attr(attr);
    let mut iter = item.clone().into_iter();
    iter.by_ref().find(|t| {
        if let TokenTree::Ident(ident) = t {
            ident == "fn"
        } else {
            false
        }
    });
    let Some(TokenTree::Ident(native_name)) = iter.next() else {
        return None;
    };
    let mut result: TokenStream = AttachDocIter {
        iter: item.clone().into_iter().peekable(),
        done: false,
        maps: &names,
        native_name: &native_name,
        doc_buf: None,
    }
    .collect();

    for (name, keys) in names {
        if name == native_name {
            // do something..
            continue;
        }
        let iter = item.clone().into_iter();

        let mut params = vec![];
        let mut generics = vec![];
        let mut is_async = false;

        if cfg {
            result.extend(tt_cfg(Span::call_site(), keys));
        }
        result.extend(tt_doc_hidden(Span::call_site()));
        result.extend([
            // #[inline(always)]
            TokenTree::Punct(Punct::new('#', Spacing::Joint)),
            TokenTree::Group(Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter([
                    TokenTree::Ident(Ident::new("inline", name.span())),
                    TokenTree::Group(Group::new(
                        Delimiter::Parenthesis,
                        TokenStream::from_iter([TokenTree::Ident(Ident::new(
                            "always",
                            name.span(),
                        ))]),
                    )),
                ]),
            )),
        ]);
        result.extend(RemapFnIter {
            iter: iter.peekable(),
            replace_fn_name: Some(name),
            params: Some(&mut params),
            generics: Some(&mut generics),

            is_async: &mut is_async,

            inside_sharp_punct: 0,
            scan_fn_status: ScanFnStatus::Unseen,
            last_punct: None,
            reading_generics: false,
            read_generic_now: false,
        });

        let mut block = TokenStream::new();
        if use_self {
            block.extend([
                TokenTree::Ident(Ident::new("Self", native_name.span())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            ]);
        }
        block.extend([TokenTree::Ident(native_name.clone())]);
        if !generics.is_empty() {
            block.extend([
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            ]);
            for (punct, ident) in generics {
                if let Some(punct) = punct
                    && punct.spacing() == Spacing::Joint
                {
                    // sometimes we need to elide the lifetime
                    continue;
                }
                block.extend([
                    TokenTree::Ident(ident),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                ]);
            }
            block.extend([TokenTree::Punct(Punct::new('>', Spacing::Alone))]);
        }
        block.extend([TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from_iter(params.into_iter().flat_map(|ident| {
                [
                    TokenTree::Ident(ident),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                ]
            })),
        ))]);

        if is_async {
            block.extend([
                TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                TokenTree::Ident(Ident::new("await", Span::call_site())),
            ]);
        }

        result.extend([TokenTree::Group(Group::new(Delimiter::Brace, block))]);
    }
    Some(result)
}

fn tt_doc_hidden(span: Span) -> [TokenTree; 2] {
    [
        TokenTree::Punct(Punct::new('#', Spacing::Joint)),
        TokenTree::Group(Group::new(
            Delimiter::Bracket,
            TokenStream::from_iter([
                TokenTree::Ident(Ident::new("doc", span)),
                TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    TokenStream::from_iter([TokenTree::Ident(Ident::new("hidden", span))]),
                )),
            ]),
        )),
    ]
}

fn tt_allow_lint(lint: &str, span: Span) -> [TokenTree; 2] {
    [
        TokenTree::Punct(Punct::new('#', Spacing::Joint)),
        TokenTree::Group(Group::new(
            Delimiter::Bracket,
            TokenStream::from_iter([
                TokenTree::Ident(Ident::new("allow", span)),
                TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    TokenStream::from_iter([TokenTree::Ident(Ident::new(lint, span))]),
                )),
            ]),
        )),
    ]
}

const CFG_KEY_NAME: &str = "rc_mapping";

fn tt_cfg<I>(span: Span, iter: I) -> [TokenTree; 2]
where
    I: IntoIterator<Item = Ident>,
{
    // let [a0, a1] = tt_allow_lint("unexpected_cfgs", span);
    [
        TokenTree::Punct(Punct::new('#', Spacing::Joint)),
        TokenTree::Group(Group::new(
            Delimiter::Bracket,
            TokenStream::from_iter([
                TokenTree::Ident(Ident::new("cfg", span)),
                TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    TokenStream::from_iter([
                        TokenTree::Ident(Ident::new("any", span)),
                        TokenTree::Group(Group::new(
                            Delimiter::Parenthesis,
                            TokenStream::from_iter(iter.into_iter().flat_map(|ident| {
                                [
                                    TokenTree::Ident(Ident::new(CFG_KEY_NAME, ident.span())),
                                    TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                                    TokenTree::Literal(Literal::string(&ident.to_smolstr())),
                                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                                ]
                            })),
                        )),
                    ]),
                )),
            ]),
        )),
    ]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScanFnStatus {
    Unseen,
    Seen,
    Passed,
}

struct RemapFnIter<'a, I: Iterator> {
    iter: Peekable<I>,
    replace_fn_name: Option<Ident>,
    params: Option<&'a mut Vec<Ident>>,
    generics: Option<&'a mut Vec<(Option<Punct>, Ident)>>,

    is_async: &'a mut bool,

    scan_fn_status: ScanFnStatus,
    last_punct: Option<Punct>,
    reading_generics: bool,
    read_generic_now: bool,
    inside_sharp_punct: u16,
}

impl<I: Iterator<Item = TokenTree>> Iterator for RemapFnIter<'_, I> {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.iter.next()?;
        // strip trailing block (or ';')
        let peek = self.iter.peek()?;

        if let TokenTree::Punct(punct) = &current {
            if self
                .last_punct
                .as_ref()
                .is_none_or(|p| p.spacing() == Spacing::Alone || matches!(p.as_char(), '>' | '<'))
            {
                match punct.as_char() {
                    '<' => self.inside_sharp_punct += 1,
                    '>' => self.inside_sharp_punct -= 1,
                    _ => (),
                }
            }
            self.last_punct = Some(punct.clone());
        } else {
            self.last_punct = None;
        }

        if self.inside_sharp_punct == 1
            && self.reading_generics
            && let Some(ref mut generics) = self.generics
        {
            if let TokenTree::Punct(punct) = &current {
                match punct.as_char() {
                    ',' => self.read_generic_now = true,
                    '\'' => {
                        if let TokenTree::Ident(ident) = peek
                            && self.read_generic_now
                            && punct.spacing() == Spacing::Joint
                        {
                            generics.push((Some(punct.clone()), ident.clone()));
                            self.read_generic_now = false;
                        }
                    }
                    _ => (),
                }
            } else if let TokenTree::Ident(ident) = &current
                && self.read_generic_now
            {
                generics.push((None, ident.clone()));
                self.read_generic_now = false;
            }
        }
        if self.inside_sharp_punct > 0 {
            return Some(current);
        } else if self.generics.is_some() && self.reading_generics {
            self.reading_generics = false;
            self.generics = None;
        }

        // replace function name
        if self.scan_fn_status == ScanFnStatus::Unseen
            && let TokenTree::Ident(ident) = &current
        {
            if ident == "fn" {
                self.scan_fn_status = ScanFnStatus::Seen;
            } else if ident == "async" {
                *self.is_async = true;
            }
        } else if self.scan_fn_status == ScanFnStatus::Seen
            && let TokenTree::Ident(_) = &current
        {
            current = TokenTree::Ident(self.replace_fn_name.take().unwrap());
            self.scan_fn_status = ScanFnStatus::Passed;
            if let TokenTree::Punct(punct) = peek
                && punct.as_char() == '<'
                && punct.spacing() == Spacing::Alone
            {
                self.reading_generics = true;
                self.read_generic_now = true;
            }
        }

        if let TokenTree::Group(group) = &current
            && group.delimiter() == Delimiter::Parenthesis
            && let Some(params) = self.params.take()
        {
            let mut iter = group.stream().into_iter();
            let mut last_punct = None;
            while let Some(next) = iter.next() {
                let TokenTree::Ident(ident) = next else {
                    if let TokenTree::Punct(punct) = next {
                        last_punct = Some(punct);
                    } else {
                        last_punct = None;
                    }
                    // panic!("non-ident pattern in function arguments is not supported")
                    continue;
                };
                if last_punct
                    .as_ref()
                    .is_some_and(|p| p.spacing() == Spacing::Joint)
                    || ident == "mut"
                {
                    last_punct = None;
                    continue;
                }

                params.push(ident);

                // forward to next ident
                let mut inside_sharps = 0u8;
                let mut last_spacing = Spacing::Alone;
                iter.by_ref().find(|t| {
                    let mut is_comma = false;
                    if let TokenTree::Punct(punct) = t {
                        if punct.spacing() == Spacing::Alone && last_spacing == Spacing::Alone {
                            match punct.as_char() {
                                '<' => inside_sharps += 1,
                                '>' => inside_sharps -= 1,
                                ',' => is_comma = true,
                                _ => (),
                            }
                        }
                        last_spacing = punct.spacing();
                    }

                    inside_sharps == 0 && is_comma
                });
            }
            self.params = None;
        }

        Some(current)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

struct AttachDocIter<'a, I: Iterator> {
    iter: Peekable<I>,
    done: bool,
    maps: &'a HashMap<Ident, Vec<Ident>>,
    native_name: &'a Ident,

    doc_buf: Option<String>,
}

impl<I: Iterator<Item = TokenTree>> Iterator for AttachDocIter<'_, I> {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(doc) = self.doc_buf.take() {
            return Some(TokenTree::Group(Group::new(
                Delimiter::Bracket,
                [
                    TokenTree::Ident(Ident::new("doc", Span::call_site())),
                    TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                    TokenTree::Literal(Literal::string(&doc)),
                ]
                .into_iter()
                .collect(),
            )));
        }

        if !self.done
            && self.iter.peek().is_some_and(|t| {
                (!if let TokenTree::Group(g) = t
                    && g.delimiter() == Delimiter::Bracket
                {
                    true
                } else {
                    false
                }) && (!if let TokenTree::Punct(p) = t
                    && p.as_char() == '#'
                {
                    true
                } else {
                    false
                })
            })
        {
            self.done = true;
            if self.maps.len() > 1 || !self.maps.contains_key(self.native_name) {
                // have something meaningful
                let mut doc = "\n\n_Mapped names: ".to_owned();
                let mut first = true;
                for (k, maps) in self.maps.iter().filter(|(k, _)| k != &self.native_name) {
                    let maps_desc = maps.iter().fold(String::new(), |mut s, i| {
                        if s.is_empty() {
                            i.to_string()
                        } else {
                            write!(&mut s, ", {}", i).unwrap();
                            s
                        }
                    });
                    if first {
                        write!(&mut doc, "`{}` ({})", k, maps_desc).unwrap();
                    } else {
                        write!(&mut doc, ", `{}` ({})", k, maps_desc).unwrap();
                    }
                    first = false;
                }
                doc.push_str("._"); // trailing italic mark

                self.doc_buf = Some(doc);
                return Some(TokenTree::Punct(Punct::new('#', Spacing::Joint)));
            }
        }
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (floor, ceil) = self.iter.size_hint();
        (floor, ceil.map(|len| len + 2))
    }
}

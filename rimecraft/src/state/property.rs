pub struct Property<T, D: PropertyData<T>> {
    name: String,
    data: D,
    _type: std::marker::PhantomData<T>,
}

impl<T, D: PropertyData<T>> Property<T, D> {
    pub fn new(name: String, data: D::Params) -> Self {
        Self {
            data: D::new(&name, data),
            name,
            _type: std::marker::PhantomData,
        }
    }
}

pub type EmptyData<T> = std::marker::PhantomData<T>;

pub trait PropertyData<T> {
    type Params;

    fn new(name: &str, params: Self::Params) -> Self;

    fn values(&self) -> Vec<T>;
    fn parse(&self, str: &str) -> Option<T>;
    fn name(&self, value: T) -> String;
}

pub struct IntPropertyData {
    min: i32,
    max: i32,
}

impl PropertyData<i32> for IntPropertyData {
    type Params = (i32, i32);

    fn new(name: &str, (mut min, mut max): Self::Params) -> Self {
        if min < 0 {
            min = 0;
            tracing::error!("Min value of {name} must be 0 or greater");
        }
        if max <= min {
            max = min + 1;
            tracing::error!("Max value of {name} should be greater than min (\"{min}\")");
        }
        Self { min, max }
    }

    fn values(&self) -> Vec<i32> {
        (self.min..=self.max).into_iter().collect()
    }

    fn parse(&self, str: &str) -> Option<i32> {
        str.parse::<i32>().map_or(None, |e| {
            if e >= self.min && e <= self.max {
                Some(e)
            } else {
                None
            }
        })
    }

    fn name(&self, value: i32) -> String {
        value.to_string()
    }
}

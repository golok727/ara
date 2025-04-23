pub struct Polygon<'a, T> {
    pub points: &'a [T],
    pub closed: bool,
}

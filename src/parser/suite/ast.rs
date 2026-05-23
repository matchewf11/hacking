#[derive(PartialEq, Debug)]
pub struct Suite(pub Vec<Group>);

#[derive(PartialEq, Debug)]
pub struct Group {
    pub name: String,
    pub tests: Vec<Test>,
}

#[derive(PartialEq, Debug)]
pub struct Test {
    pub name: String,
    pub value: String,
    pub asserts: Vec<Assert>,
}

#[derive(PartialEq, Debug)]
pub struct Assert {
    pub negated: bool,
    pub target: AssertTarget,
    pub matcher: Option<Matcher>,
}

#[derive(PartialEq, Debug)]
pub enum AssertTarget {
    /// `assert status <code>`
    Status,
    /// `assert body` or `assert body.<path>`
    Body(BodyTarget),
    /// `assert header.<name>`
    Header(String),
    /// `assert cookie.<name>` or `assert cookie.<name>.<attribute>`
    Cookie(String, Option<String>),
    /// `assert duration`
    Duration,
}

#[derive(PartialEq, Debug)]
pub enum BodyTarget {
    Raw,
    Path(Vec<PathSegment>),
}

#[derive(PartialEq, Debug)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

#[derive(PartialEq, Debug)]
pub enum Matcher {
    // Unary
    Present,
    Absent,
    Empty,
    // Binary
    Equals(AssertValue),
    NotEquals(AssertValue),
    Contains(AssertValue),
    NotContains(AssertValue),
    StartsWith(AssertValue),
    EndsWith(AssertValue),
    Gt(AssertValue),
    Gte(AssertValue),
    Lt(AssertValue),
    Lte(AssertValue),
    Length(LengthArg),
    Matches(String),
}

/// `assert body.items length 5` vs `assert body.tags length gt 0`
#[derive(PartialEq, Debug)]
pub enum LengthArg {
    Exact(AssertValue),
    Gt(AssertValue),
    Gte(AssertValue),
    Lt(AssertValue),
    Lte(AssertValue),
}

#[derive(PartialEq, Debug)]
pub enum AssertValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Regex(String),
}

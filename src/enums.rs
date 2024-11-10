#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Integer,
    Real,
    Char,
    String,
    Date,
    Array(Box<Array>),
    // Composite,
    // ENUM
    // Pointers
}

#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    pub t: Box<VariableType>,
    pub lower: i64,
    pub upper: i64,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MOperator {Plus, Minus, Mul, Div, IntDiv, Mod}

#[derive(Debug, PartialEq)]
pub enum Comparator {Eq, Neq, Gt, Lt, Gte, Lte }
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum HirExpr {
    Number(f64),
    Boolean(bool),
    String(String),
    Null,
    Undefined,
    Identifier(String),

    Binary { op: BinOp, left: Box<HirExpr>, right: Box<HirExpr> },
    Unary { op: UnaryOp, operand: Box<HirExpr> },
    Typeof(Box<HirExpr>),

    Call { callee: Box<HirExpr>, args: Vec<HirExpr> },

    Ternary { cond: Box<HirExpr>, then_expr: Box<HirExpr>, else_expr: Box<HirExpr> },

    Function { name: String, params: Vec<String>, body: Vec<HirExpr> },

    Return(Option<Box<HirExpr>>),
    Var { name: String, init: Option<Box<HirExpr>>, #[allow(dead_code)] is_mut: bool },

    If { cond: Box<HirExpr>, then_body: Vec<HirExpr>, else_body: Option<Vec<HirExpr>> },
    While { cond: Box<HirExpr>, body: Vec<HirExpr> },
    For { init: Option<Box<HirExpr>>, cond: Option<Box<HirExpr>>, update: Option<Box<HirExpr>>, body: Vec<HirExpr> },

    Block(Vec<HirExpr>),

    Break,
    Continue,

    Assign { target: Box<HirExpr>, value: Box<HirExpr> },

    Array(Vec<HirExpr>),
    Index { object: Box<HirExpr>, index: Box<HirExpr> },

    Object(Vec<(String, HirExpr)>),
    Property { object: Box<HirExpr>, name: String },
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum UnaryOp {
    Not, Neg, PreInc, PreDec, PostInc, PostDec,
}

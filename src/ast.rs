use std::collections::HashMap;
use std::sync::Arc;
use cel_parser::{ArithmeticOp, Atom, Expression, Member, RelationOp, UnaryOp};
use cel_parser::Member::{Attribute, Fields, Index};
use serde::{Deserialize, Serialize};
use crate::{HostContext};
use crate::models::{PassableMap, PassableValue};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(crate) struct ASTExecutionContext {
    pub(crate) variables: PassableMap,
    pub(crate) expression: JSONExpression,
    pub(crate) platform: Option<HashMap<String, Vec<PassableValue>>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JSONRelationOp {
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Equals,
    NotEquals,
    In,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JSONArithmeticOp {
    Add,
    Subtract,
    Divide,
    Multiply,
    Modulus,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JSONUnaryOp {
    Not,
    DoubleNot,
    Minus,
    DoubleMinus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JSONExpression {
    Arithmetic(Box<JSONExpression>, JSONArithmeticOp, Box<JSONExpression>),
    Relation(Box<JSONExpression>, JSONRelationOp, Box<JSONExpression>),
    Ternary(Box<JSONExpression>, Box<JSONExpression>, Box<JSONExpression>),
    Or(Box<JSONExpression>, Box<JSONExpression>),
    And(Box<JSONExpression>, Box<JSONExpression>),
    Unary(JSONUnaryOp, Box<JSONExpression>),
    Member(Box<JSONExpression>, Box<JSONMember>),
    FunctionCall(Box<JSONExpression>, Option<Box<JSONExpression>>, Vec<JSONExpression>),
    List(Vec<JSONExpression>),
    Map(Vec<(JSONExpression, JSONExpression)>),
    Atom(JSONAtom),
    Ident(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JSONMember {
    Attribute(String),
    Index(Box<JSONExpression>),
    Fields(Vec<(String, JSONExpression)>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JSONAtom {
    Int(i64),
    UInt(u64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Bool(bool),
    Null,
}

// Conversion functions
impl From<JSONRelationOp> for RelationOp {
    fn from(op: JSONRelationOp) -> Self {
        match op {
            JSONRelationOp::LessThan => RelationOp::LessThan,
            JSONRelationOp::LessThanEq => RelationOp::LessThanEq,
            JSONRelationOp::GreaterThan => RelationOp::GreaterThan,
            JSONRelationOp::GreaterThanEq => RelationOp::GreaterThanEq,
            JSONRelationOp::Equals => RelationOp::Equals,
            JSONRelationOp::NotEquals => RelationOp::NotEquals,
            JSONRelationOp::In => RelationOp::In,
        }
    }
}

impl From<JSONArithmeticOp> for ArithmeticOp {
    fn from(op: JSONArithmeticOp) -> Self {
        match op {
            JSONArithmeticOp::Add => ArithmeticOp::Add,
            JSONArithmeticOp::Subtract => ArithmeticOp::Subtract,
            JSONArithmeticOp::Divide => ArithmeticOp::Divide,
            JSONArithmeticOp::Multiply => ArithmeticOp::Multiply,
            JSONArithmeticOp::Modulus => ArithmeticOp::Modulus,
        }
    }
}

impl From<JSONUnaryOp> for UnaryOp {
    fn from(op: JSONUnaryOp) -> Self {
        match op {
            JSONUnaryOp::Not => UnaryOp::Not,
            JSONUnaryOp::DoubleNot => UnaryOp::DoubleNot,
            JSONUnaryOp::Minus => UnaryOp::Minus,
            JSONUnaryOp::DoubleMinus => UnaryOp::DoubleMinus,
        }
    }
}

impl From<JSONExpression> for Expression {
    fn from(expr: JSONExpression) -> Self {
        match expr {
            JSONExpression::Arithmetic(left, op, right) => Expression::Arithmetic(
                Box::new((*left).into()),
                op.into(),
                Box::new((*right).into()),
            ),
            JSONExpression::Relation(left, op, right) => Expression::Relation(
                Box::new((*left).into()),
                op.into(),
                Box::new((*right).into()),
            ),
            JSONExpression::Ternary(cond, true_expr, false_expr) => Expression::Ternary(
                Box::new((*cond).into()),
                Box::new((*true_expr).into()),
                Box::new((*false_expr).into()),
            ),
            JSONExpression::Or(left, right) => Expression::Or(
                Box::new((*left).into()),
                Box::new((*right).into()),
            ),
            JSONExpression::And(left, right) => Expression::And(
                Box::new((*left).into()),
                Box::new((*right).into()),
            ),
            JSONExpression::Unary(op, expr) => Expression::Unary(
                op.into(),
                Box::new((*expr).into()),
            ),
            JSONExpression::Member(expr, member) => Expression::Member(
                Box::new((*expr).into()),
                Box::new((*member).into()),
            ),
            JSONExpression::FunctionCall(func, optional_expr, args) => Expression::FunctionCall(
                Box::new((*func).into()),
                optional_expr.map(|e| Box::new((*e).into())),
                args.into_iter().map(|e| e.into()).collect(),
            ),
            JSONExpression::List(items) => Expression::List(
                items.into_iter().map(|e| e.into()).collect(),
            ),
            JSONExpression::Map(items) => Expression::Map(
                items.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            ),
            JSONExpression::Atom(atom) => Expression::Atom(atom.into()),
            JSONExpression::Ident(s) => Expression::Ident(Arc::new(s)),
        }
    }
}

impl From<JSONMember> for cel_parser::Member {
    fn from(member: JSONMember) -> Self {
        match member {
            JSONMember::Attribute(s) => Attribute(Arc::new(s)),
            JSONMember::Index(expr) => Index(Box::new((*expr).into())),
            JSONMember::Fields(fields) => Fields(
                fields.into_iter().map(|(k, v)| (Arc::new(k), v.into())).collect(),
            ),
        }
    }
}

impl From<JSONAtom> for Atom {
    fn from(atom: JSONAtom) -> Self {
        match atom {
            JSONAtom::Int(i) => Atom::Int(i),
            JSONAtom::UInt(u) => Atom::UInt(u),
            JSONAtom::Float(f) => Atom::Float(f),
            JSONAtom::String(s) => Atom::String(Arc::new(s)),
            JSONAtom::Bytes(b) => Atom::Bytes(Arc::new(b)),
            JSONAtom::Bool(b) => Atom::Bool(b),
            JSONAtom::Null => Atom::Null,
        }
    }
}

impl From<Expression> for JSONExpression {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::Arithmetic(left, op, right) => JSONExpression::Arithmetic(
                Box::new((*left).into()),
                op.into(),
                Box::new((*right).into()),
            ),
            Expression::Relation(left, op, right) => JSONExpression::Relation(
                Box::new((*left).into()),
                op.into(),
                Box::new((*right).into()),
            ),
            Expression::Ternary(cond, true_expr, false_expr) => JSONExpression::Ternary(
                Box::new((*cond).into()),
                Box::new((*true_expr).into()),
                Box::new((*false_expr).into()),
            ),
            Expression::Or(left, right) => JSONExpression::Or(
                Box::new((*left).into()),
                Box::new((*right).into()),
            ),
            Expression::And(left, right) => JSONExpression::And(
                Box::new((*left).into()),
                Box::new((*right).into()),
            ),
            Expression::Unary(op, expr) => JSONExpression::Unary(
                op.into(),
                Box::new((*expr).into()),
            ),
            Expression::Member(expr, member) => JSONExpression::Member(
                Box::new((*expr).into()),
                Box::new((*member).into()),
            ),
            Expression::FunctionCall(func, optional_expr, args) => JSONExpression::FunctionCall(
                Box::new((*func).into()),
                optional_expr.map(|e| Box::new((*e).into())),
                args.into_iter().map(|e| e.into()).collect(),
            ),
            Expression::List(items) => JSONExpression::List(
                items.into_iter().map(|e| e.into()).collect(),
            ),
            Expression::Map(items) => JSONExpression::Map(
                items.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            ),
            Expression::Atom(atom) => JSONExpression::Atom(atom.into()),
            Expression::Ident(s) => JSONExpression::Ident((*s).clone()),
        }
    }
}

// Implement From for other types
impl From<RelationOp> for JSONRelationOp {
    fn from(op: RelationOp) -> Self {
        match op {
            RelationOp::LessThan => JSONRelationOp::LessThan,
            RelationOp::LessThanEq => JSONRelationOp::LessThanEq,
            RelationOp::GreaterThan => JSONRelationOp::GreaterThan,
            RelationOp::GreaterThanEq => JSONRelationOp::GreaterThanEq,
            RelationOp::Equals => JSONRelationOp::Equals,
            RelationOp::NotEquals => JSONRelationOp::NotEquals,
            RelationOp::In => JSONRelationOp::In,
        }
    }
}

impl From<ArithmeticOp> for JSONArithmeticOp {
    fn from(op: ArithmeticOp) -> Self {
        match op {
            ArithmeticOp::Add => JSONArithmeticOp::Add,
            ArithmeticOp::Subtract => JSONArithmeticOp::Subtract,
            ArithmeticOp::Divide => JSONArithmeticOp::Divide,
            ArithmeticOp::Multiply => JSONArithmeticOp::Multiply,
            ArithmeticOp::Modulus => JSONArithmeticOp::Modulus,
        }
    }
}

impl From<UnaryOp> for JSONUnaryOp {
    fn from(op: UnaryOp) -> Self {
        match op {
            UnaryOp::Not => JSONUnaryOp::Not,
            UnaryOp::DoubleNot => JSONUnaryOp::DoubleNot,
            UnaryOp::Minus => JSONUnaryOp::Minus,
            UnaryOp::DoubleMinus => JSONUnaryOp::DoubleMinus,
        }
    }
}

impl From<Member> for JSONMember {
    fn from(member: Member) -> Self {
        match member {
            Attribute(s) => JSONMember::Attribute((*s).clone()),
            Index(expr) => JSONMember::Index(Box::new((*expr).into())),
            Fields(fields) => JSONMember::Fields(
                fields.into_iter().map(|(k, v)| ((*k).clone(), v.into())).collect(),
            ),
        }
    }
}

impl From<Atom> for JSONAtom {
    fn from(atom: Atom) -> Self {
        match atom {
            Atom::Int(i) => JSONAtom::Int(i),
            Atom::UInt(u) => JSONAtom::UInt(u),
            Atom::Float(f) => JSONAtom::Float(f),
            Atom::String(s) => JSONAtom::String((*s).clone()),
            Atom::Bytes(b) => JSONAtom::Bytes((*b).clone()),
            Atom::Bool(b) => JSONAtom::Bool(b),
            Atom::Null => JSONAtom::Null,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_serializing() {
            // ((5 + 3) > 7) && (name.length() in [5, 10, 15])
            let expr = Expression::And(
                Box::new(Expression::Relation(
                    Box::new(Expression::Arithmetic(
                        Box::new(Expression::Atom(Atom::Int(5))),
                        ArithmeticOp::Add,
                        Box::new(Expression::Atom(Atom::Int(3)))
                    )),
                    RelationOp::GreaterThan,
                    Box::new(Expression::Atom(Atom::Int(7)))
                )),
                Box::new(Expression::Relation(
                    Box::new(Expression::FunctionCall(
                        Box::new(Expression::Member(
                            Box::new(Expression::Ident(Arc::new("name".to_string()))),
                            Box::new(Attribute(Arc::new("length".to_string())))
                        )),
                        None,
                        vec![]
                    )),
                    RelationOp::In,
                    Box::new(Expression::List(vec![
                        Expression::Atom(Atom::Int(5)),
                        Expression::Atom(Atom::Int(10)),
                        Expression::Atom(Atom::Int(15))
                    ]))
                ))
            );

            // Convert to JSONExpression
            let json_expr: JSONExpression = expr.clone().into();

            // Serialize to JSON
            let json_string = serde_json::to_string_pretty(&json_expr).unwrap();

            println!("JSON representation:");
            println!("{}", json_string);

            // Deserialize back to JSONExpression
            let deserialized_json_expr: JSONExpression = serde_json::from_str(&json_string).unwrap();

            // Convert back to original Expression
            let deserialized_expr: Expression = deserialized_json_expr.into();

            println!("\nDeserialized Expression:");
            println!("{:?}", deserialized_expr);

            // Check if the original and deserialized expressions are equal
            assert_eq!(expr, deserialized_expr);
            println!("\nOriginal and deserialized expressions are equal!");
    }
}

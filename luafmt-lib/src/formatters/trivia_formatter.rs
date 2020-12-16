use full_moon::ast::{
    punctuated::{Pair, Punctuated},
    span::ContainedSpan,
    Assignment, Call, Expression, FunctionArgs, FunctionBody, FunctionCall, Index, LocalAssignment,
    MethodCall, Suffix, TableConstructor, Value, Var, VarExpression,
};
use full_moon::tokenizer::{Token, TokenReference};
use std::borrow::Cow;

// Special Case for Statements
pub fn assignment_add_trivia<'ast>(
    assignment: Assignment<'ast>,
    leading_trivia: Vec<Token<'ast>>,
    trailing_trivia: Vec<Token<'ast>>,
) -> Assignment<'ast> {
    // TODO: Add leading trivia
    let mut formatted_expression_list = assignment.expr_list().to_owned();

    // Retrieve last item and add new line to it
    if let Some(last_pair) = formatted_expression_list.pop() {
        match last_pair {
            Pair::End(value) => {
                let expression = expression_add_trailing_trivia(value, trailing_trivia);
                formatted_expression_list.push(Pair::End(expression));
            }
            Pair::Punctuated(_, _) => (), // TODO: Is it possible for this to happen? Do we need to account for it?
        }
    }

    assignment.with_expr_list(formatted_expression_list)
}

pub fn function_call_add_trivia<'ast>(
    function_call: FunctionCall<'ast>,
    leading_trivia: Vec<Token<'ast>>,
    trailing_trivia: Vec<Token<'ast>>,
) -> FunctionCall<'ast> {
    function_call_add_trailing_trivia(function_call, trailing_trivia)
}

pub fn local_assignment_add_trivia<'ast>(
    local_assignment: LocalAssignment<'ast>,
    leading_trivia: Vec<Token<'ast>>,
    trailing_trivia: Vec<Token<'ast>>,
) -> LocalAssignment<'ast> {
    // TODO: Add trailing trivia
    // Add newline at the end of LocalAssignment expression list
    // Expression list should already be formatted
    let mut formatted_expression_list = local_assignment.expr_list().to_owned();

    // Retrieve last item and add new line to it
    if let Some(last_pair) = formatted_expression_list.pop() {
        match last_pair {
            Pair::End(value) => {
                let expression = expression_add_trailing_trivia(value, trailing_trivia);
                formatted_expression_list.push(Pair::End(expression));
            }
            Pair::Punctuated(_, _) => (), // TODO: Is it possible for this to happen? Do we need to account for it?
        }
    }

    local_assignment.with_expr_list(formatted_expression_list)
}

// Remainder of Nodes

/// Adds trailing trivia at the end of a ContainedSpan node
pub fn contained_span_add_trailing_trivia<'ast>(
    contained_span: ContainedSpan<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> ContainedSpan<'ast> {
    let (start_token, end_token) = contained_span.tokens();
    ContainedSpan::new(
        Cow::Owned(start_token.to_owned()),
        Cow::Owned(token_reference_add_trailing_trivia(
            end_token.to_owned(),
            trailing_trivia,
        )),
    )
}

/// Adds trailing trivia at the end of a Call node
pub fn call_add_trailing_trivia<'ast>(
    call: Call<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> Call<'ast> {
    match call {
        Call::AnonymousCall(function_args) => Call::AnonymousCall(
            function_args_add_trailing_trivia(function_args, trailing_trivia),
        ),
        Call::MethodCall(method_call) => Call::MethodCall(method_call_add_trailing_trivia(
            method_call,
            trailing_trivia,
        )),
    }
}

/// Adds traviling trivia at the end of an Expression node
pub fn expression_add_trailing_trivia<'ast>(
    expression: Expression<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> Expression<'ast> {
    match expression {
        Expression::Value { value, binop } => Expression::Value {
            value: Box::new(value_add_trailing_trivia(*value, trailing_trivia)),
            binop,
        },

        // Add trailing trivia to the end of parentheses
        Expression::Parentheses {
            contained,
            expression,
        } => Expression::Parentheses {
            contained: contained_span_add_trailing_trivia(contained, trailing_trivia),
            expression,
        },

        // Keep recursing down until we find an Expression::Value
        Expression::UnaryOperator { unop, expression } => Expression::UnaryOperator {
            unop,
            expression: Box::new(expression_add_trailing_trivia(*expression, trailing_trivia)),
        },
    }
}

/// Adds trailing trivia at the end of a FunctinoArgs node
pub fn function_args_add_trailing_trivia<'ast>(
    function_args: FunctionArgs<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> FunctionArgs<'ast> {
    match function_args {
        FunctionArgs::Parentheses {
            parentheses,
            arguments,
        } => FunctionArgs::Parentheses {
            parentheses: contained_span_add_trailing_trivia(parentheses, trailing_trivia),
            arguments,
        },

        // Add for completeness
        FunctionArgs::String(token_reference) => FunctionArgs::String(Cow::Owned(
            token_reference_add_trailing_trivia(token_reference.into_owned(), trailing_trivia),
        )),
        FunctionArgs::TableConstructor(table_constructor) => FunctionArgs::TableConstructor(
            table_constructor_add_trailing_trivia(table_constructor, trailing_trivia),
        ),
    }
}

/// Adds trailing trivia at the end of a FunctionBody node
pub fn function_body_add_trailing_trivia<'ast>(
    function_body: FunctionBody<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> FunctionBody<'ast> {
    let function_body_token = function_body.end_token().to_owned();
    function_body.with_end_token(Cow::Owned(token_reference_add_trailing_trivia(
        function_body_token,
        trailing_trivia,
    )))
}

/// Adds trailing trivia at the end of a FunctionCall node
pub fn function_call_add_trailing_trivia<'ast>(
    function_call: FunctionCall<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> FunctionCall<'ast> {
    let mut new_suffixes: Vec<Suffix<'ast>> = function_call
        .iter_suffixes()
        .map(|x| x.to_owned())
        .collect();
    if let Some(last_suffix) = new_suffixes.pop() {
        new_suffixes.push(suffix_add_trailing_trivia(
            last_suffix.to_owned(),
            trailing_trivia,
        ))
    }

    function_call.with_suffixes(new_suffixes)
}

/// Adds trailing trivia at the end of an Index node
pub fn index_add_trailing_trivia<'ast>(
    index: Index<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> Index<'ast> {
    match index {
        Index::Brackets {
            brackets,
            expression,
        } => Index::Brackets {
            brackets: contained_span_add_trailing_trivia(brackets, trailing_trivia),
            expression,
        },
        Index::Dot { dot, name } => Index::Dot {
            dot,
            name: Cow::Owned(token_reference_add_trailing_trivia(
                name.into_owned(),
                trailing_trivia,
            )),
        },
    }
}

/// Adds trailing trivia at the end of a MethodCall node
pub fn method_call_add_trailing_trivia<'ast>(
    method_call: MethodCall<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> MethodCall<'ast> {
    let method_call_args = method_call.args().to_owned();
    method_call.with_args(function_args_add_trailing_trivia(
        method_call_args,
        trailing_trivia,
    ))
}

/// Adds trailing trivia at the end of a Suffix node
pub fn suffix_add_trailing_trivia<'ast>(
    suffix: Suffix<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> Suffix<'ast> {
    match suffix {
        Suffix::Call(call) => Suffix::Call(call_add_trailing_trivia(call, trailing_trivia)),
        Suffix::Index(index) => Suffix::Index(index_add_trailing_trivia(index, trailing_trivia)),
    }
}

/// Adds trailing trivia at the end of a TableConstructor node
pub fn table_constructor_add_trailing_trivia<'ast>(
    table_constructor: TableConstructor<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> TableConstructor<'ast> {
    let table_constructor_braces = table_constructor.braces().to_owned();
    table_constructor.with_braces(contained_span_add_trailing_trivia(
        table_constructor_braces,
        trailing_trivia,
    ))
}

/// Adds trailing trivia at the end of a TokenReference node
pub fn token_reference_add_trailing_trivia<'ast>(
    token_reference: TokenReference<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> TokenReference<'ast> {
    let leading_trivia = token_reference
        .leading_trivia()
        .map(|x| x.to_owned())
        .collect();
    TokenReference::new(
        leading_trivia,
        token_reference.token().to_owned(),
        trailing_trivia,
    )
}

/// Adds trailing trivia at the end of a Value node
pub fn value_add_trailing_trivia<'ast>(
    value: Value<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> Value<'ast> {
    match value {
        Value::String(token_reference) => Value::String(Cow::Owned(
            token_reference_add_trailing_trivia(token_reference.into_owned(), trailing_trivia),
        )),
        Value::Number(token_reference) => Value::Number(Cow::Owned(
            token_reference_add_trailing_trivia(token_reference.into_owned(), trailing_trivia),
        )),
        Value::Symbol(token_reference) => Value::Symbol(Cow::Owned(
            token_reference_add_trailing_trivia(token_reference.into_owned(), trailing_trivia),
        )),
        Value::ParseExpression(expression) => {
            Value::ParseExpression(expression_add_trailing_trivia(expression, trailing_trivia))
        }
        Value::FunctionCall(function_call) => Value::FunctionCall(
            function_call_add_trailing_trivia(function_call, trailing_trivia),
        ),
        Value::TableConstructor(table_constructor) => Value::TableConstructor(
            table_constructor_add_trailing_trivia(table_constructor, trailing_trivia),
        ),
        Value::Var(var) => Value::Var(var_add_trailing_trivia(var, trailing_trivia)),
        Value::Function((token, function_body)) => Value::Function((
            token,
            function_body_add_trailing_trivia(function_body, trailing_trivia),
        )),
    }
}

/// Adds trailing trivia at the end of a Var node
pub fn var_add_trailing_trivia<'ast>(
    var: Var<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> Var<'ast> {
    match var {
        Var::Name(token_reference) => Var::Name(Cow::Owned(token_reference_add_trailing_trivia(
            token_reference.into_owned(),
            trailing_trivia,
        ))),
        Var::Expression(var_expression) => Var::Expression(var_expression_add_trailing_trivia(
            var_expression,
            trailing_trivia,
        )),
    }
}

/// Adds trailing trivia at the end of a VarExpression node
pub fn var_expression_add_trailing_trivia<'ast>(
    var_expression: VarExpression<'ast>,
    trailing_trivia: Vec<Token<'ast>>,
) -> VarExpression<'ast> {
    // TODO: This is copied from FunctionCall, can we combine them?
    let mut new_suffixes: Vec<Suffix<'ast>> = var_expression
        .iter_suffixes()
        .map(|x| x.to_owned())
        .collect();
    if let Some(last_suffix) = new_suffixes.pop() {
        new_suffixes.push(suffix_add_trailing_trivia(
            last_suffix.to_owned(),
            trailing_trivia,
        ))
    }

    var_expression.with_suffixes(new_suffixes)
}

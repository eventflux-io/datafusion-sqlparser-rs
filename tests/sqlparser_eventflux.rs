// Licensed under the Apache License, Version 2.0
// Tests for EventFlux CEP pattern processing AST types

use sqlparser::ast::{
    Ident, ObjectName, ObjectNamePart, PatternArrayIndex, PatternExpression, PatternLogicalOp,
    PatternMode, PatternOutputType, TableAlias, TableFactor, WithinConstraint,
};

// Helper to create ObjectName from a simple string
fn obj_name(name: &str) -> ObjectName {
    ObjectName(vec![ObjectNamePart::Identifier(Ident::new(name))])
}

// ============================================================================
// PatternMode Tests
// ============================================================================

#[test]
fn test_pattern_mode_display() {
    assert_eq!(PatternMode::Pattern.to_string(), "PATTERN");
    assert_eq!(PatternMode::Sequence.to_string(), "SEQUENCE");
}

#[test]
fn test_pattern_mode_equality() {
    assert_eq!(PatternMode::Pattern, PatternMode::Pattern);
    assert_eq!(PatternMode::Sequence, PatternMode::Sequence);
    assert_ne!(PatternMode::Pattern, PatternMode::Sequence);
}

// ============================================================================
// PatternLogicalOp Tests
// ============================================================================

#[test]
fn test_pattern_logical_op_display() {
    assert_eq!(PatternLogicalOp::And.to_string(), "AND");
    assert_eq!(PatternLogicalOp::Or.to_string(), "OR");
}

// ============================================================================
// PatternExpression Tests
// ============================================================================

#[test]
fn test_pattern_expression_stream_basic() {
    // Stream without alias: StreamName
    let expr = PatternExpression::Stream {
        alias: None,
        stream_name: obj_name("StockStream"),
        filter: None,
    };
    assert_eq!(expr.to_string(), "StockStream");
}

#[test]
fn test_pattern_expression_stream_with_alias() {
    // Stream with alias: e1=StreamName
    let expr = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("StockStream"),
        filter: None,
    };
    assert_eq!(expr.to_string(), "e1=StockStream");
}

#[test]
fn test_pattern_expression_count_exact() {
    // Exact count: A{3}
    let stream = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let expr = PatternExpression::Count {
        pattern: Box::new(stream),
        min_count: 3,
        max_count: 3,
    };
    assert_eq!(expr.to_string(), "e1=A{3}");
}

#[test]
fn test_pattern_expression_count_range() {
    // Range count: A{2,5}
    let stream = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let expr = PatternExpression::Count {
        pattern: Box::new(stream),
        min_count: 2,
        max_count: 5,
    };
    assert_eq!(expr.to_string(), "e1=A{2,5}");
}

#[test]
fn test_pattern_expression_sequence() {
    // Sequence: A -> B
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let expr = PatternExpression::Sequence {
        first: Box::new(a),
        second: Box::new(b),
    };
    assert_eq!(expr.to_string(), "e1=A -> e2=B");
}

#[test]
fn test_pattern_expression_sequence_chain() {
    // Sequence chain: A -> B -> C
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let c = PatternExpression::Stream {
        alias: Some(Ident::new("e3")),
        stream_name: obj_name("C"),
        filter: None,
    };
    // A -> B first
    let ab = PatternExpression::Sequence {
        first: Box::new(a),
        second: Box::new(b),
    };
    // Then (A -> B) -> C
    let abc = PatternExpression::Sequence {
        first: Box::new(ab),
        second: Box::new(c),
    };
    assert_eq!(abc.to_string(), "e1=A -> e2=B -> e3=C");
}

#[test]
fn test_pattern_expression_logical_and() {
    // Logical AND: A AND B
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let expr = PatternExpression::Logical {
        left: Box::new(a),
        op: PatternLogicalOp::And,
        right: Box::new(b),
    };
    assert_eq!(expr.to_string(), "e1=A AND e2=B");
}

#[test]
fn test_pattern_expression_logical_or() {
    // Logical OR: A OR B
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let expr = PatternExpression::Logical {
        left: Box::new(a),
        op: PatternLogicalOp::Or,
        right: Box::new(b),
    };
    assert_eq!(expr.to_string(), "e1=A OR e2=B");
}

#[test]
fn test_pattern_expression_every() {
    // EVERY pattern: EVERY (A -> B)
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let seq = PatternExpression::Sequence {
        first: Box::new(a),
        second: Box::new(b),
    };
    let expr = PatternExpression::Every {
        pattern: Box::new(seq),
    };
    assert_eq!(expr.to_string(), "EVERY (e1=A -> e2=B)");
}

#[test]
fn test_pattern_expression_grouped() {
    // Grouped pattern: (A -> B)
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let seq = PatternExpression::Sequence {
        first: Box::new(a),
        second: Box::new(b),
    };
    let expr = PatternExpression::Grouped {
        pattern: Box::new(seq),
    };
    assert_eq!(expr.to_string(), "(e1=A -> e2=B)");
}

// ============================================================================
// WithinConstraint Tests
// ============================================================================

#[test]
fn test_within_constraint_event_count() {
    let constraint = WithinConstraint::EventCount(100);
    assert_eq!(constraint.to_string(), "WITHIN 100 EVENTS");
}

// ============================================================================
// PatternOutputType Tests
// ============================================================================

#[test]
fn test_pattern_output_type_display() {
    assert_eq!(PatternOutputType::CurrentEvents.to_string(), "CURRENT EVENTS");
    assert_eq!(PatternOutputType::ExpiredEvents.to_string(), "EXPIRED EVENTS");
    assert_eq!(PatternOutputType::AllEvents.to_string(), "ALL EVENTS");
}

// ============================================================================
// PatternArrayIndex Tests
// ============================================================================

#[test]
fn test_pattern_array_index_display() {
    assert_eq!(PatternArrayIndex::Numeric(0).to_string(), "0");
    assert_eq!(PatternArrayIndex::Numeric(5).to_string(), "5");
    assert_eq!(PatternArrayIndex::Last.to_string(), "last");
}

#[test]
fn test_pattern_array_index_equality() {
    assert_eq!(PatternArrayIndex::Numeric(0), PatternArrayIndex::Numeric(0));
    assert_ne!(PatternArrayIndex::Numeric(0), PatternArrayIndex::Numeric(1));
    assert_eq!(PatternArrayIndex::Last, PatternArrayIndex::Last);
    assert_ne!(PatternArrayIndex::Numeric(0), PatternArrayIndex::Last);
}

// ============================================================================
// TableFactor::Pattern Tests
// ============================================================================

#[test]
fn test_table_factor_pattern_basic() {
    // FROM PATTERN (e1=A -> e2=B)
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let seq = PatternExpression::Sequence {
        first: Box::new(a),
        second: Box::new(b),
    };

    let table_factor = TableFactor::Pattern {
        mode: PatternMode::Pattern,
        pattern: seq,
        within: None,
        alias: None,
    };

    assert_eq!(table_factor.to_string(), "PATTERN (e1=A -> e2=B)");
}

#[test]
fn test_table_factor_sequence_basic() {
    // FROM SEQUENCE (e1=A -> e2=B)
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let seq = PatternExpression::Sequence {
        first: Box::new(a),
        second: Box::new(b),
    };

    let table_factor = TableFactor::Pattern {
        mode: PatternMode::Sequence,
        pattern: seq,
        within: None,
        alias: None,
    };

    assert_eq!(table_factor.to_string(), "SEQUENCE (e1=A -> e2=B)");
}

#[test]
fn test_table_factor_pattern_with_within_events() {
    // FROM PATTERN (e1=A -> e2=B) WITHIN 100 EVENTS
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let seq = PatternExpression::Sequence {
        first: Box::new(a),
        second: Box::new(b),
    };

    let table_factor = TableFactor::Pattern {
        mode: PatternMode::Pattern,
        pattern: seq,
        within: Some(WithinConstraint::EventCount(100)),
        alias: None,
    };

    assert_eq!(table_factor.to_string(), "PATTERN (e1=A -> e2=B) WITHIN 100 EVENTS");
}

#[test]
fn test_table_factor_pattern_with_alias() {
    // FROM PATTERN (e1=A -> e2=B) AS p
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let seq = PatternExpression::Sequence {
        first: Box::new(a),
        second: Box::new(b),
    };

    let table_factor = TableFactor::Pattern {
        mode: PatternMode::Pattern,
        pattern: seq,
        within: None,
        alias: Some(TableAlias {
            name: Ident::new("p"),
            columns: vec![],
        }),
    };

    assert_eq!(table_factor.to_string(), "PATTERN (e1=A -> e2=B) AS p");
}

#[test]
fn test_table_factor_pattern_count_quantifier() {
    // FROM PATTERN (e1=A{2,5} -> e2=B)
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let a_count = PatternExpression::Count {
        pattern: Box::new(a),
        min_count: 2,
        max_count: 5,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let seq = PatternExpression::Sequence {
        first: Box::new(a_count),
        second: Box::new(b),
    };

    let table_factor = TableFactor::Pattern {
        mode: PatternMode::Pattern,
        pattern: seq,
        within: None,
        alias: None,
    };

    assert_eq!(table_factor.to_string(), "PATTERN (e1=A{2,5} -> e2=B)");
}

#[test]
fn test_table_factor_pattern_every() {
    // FROM PATTERN EVERY (e1=A -> e2=B)
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let seq = PatternExpression::Sequence {
        first: Box::new(a),
        second: Box::new(b),
    };
    let every = PatternExpression::Every {
        pattern: Box::new(seq),
    };

    let table_factor = TableFactor::Pattern {
        mode: PatternMode::Pattern,
        pattern: every,
        within: None,
        alias: None,
    };

    assert_eq!(table_factor.to_string(), "PATTERN (EVERY (e1=A -> e2=B))");
}

#[test]
fn test_table_factor_pattern_logical_and() {
    // FROM PATTERN (e1=A AND e2=B) -> e3=C
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let c = PatternExpression::Stream {
        alias: Some(Ident::new("e3")),
        stream_name: obj_name("C"),
        filter: None,
    };
    let and_pattern = PatternExpression::Logical {
        left: Box::new(a),
        op: PatternLogicalOp::And,
        right: Box::new(b),
    };
    let grouped = PatternExpression::Grouped {
        pattern: Box::new(and_pattern),
    };
    let seq = PatternExpression::Sequence {
        first: Box::new(grouped),
        second: Box::new(c),
    };

    let table_factor = TableFactor::Pattern {
        mode: PatternMode::Pattern,
        pattern: seq,
        within: None,
        alias: None,
    };

    assert_eq!(table_factor.to_string(), "PATTERN ((e1=A AND e2=B) -> e3=C)");
}

// ============================================================================
// Complex Pattern Expression Tests
// ============================================================================

#[test]
fn test_complex_pattern_count_in_sequence() {
    // A{2,3} -> B{1,2} -> C
    let a = PatternExpression::Stream {
        alias: Some(Ident::new("e1")),
        stream_name: obj_name("A"),
        filter: None,
    };
    let a_count = PatternExpression::Count {
        pattern: Box::new(a),
        min_count: 2,
        max_count: 3,
    };
    let b = PatternExpression::Stream {
        alias: Some(Ident::new("e2")),
        stream_name: obj_name("B"),
        filter: None,
    };
    let b_count = PatternExpression::Count {
        pattern: Box::new(b),
        min_count: 1,
        max_count: 2,
    };
    let c = PatternExpression::Stream {
        alias: Some(Ident::new("e3")),
        stream_name: obj_name("C"),
        filter: None,
    };

    let ab = PatternExpression::Sequence {
        first: Box::new(a_count),
        second: Box::new(b_count),
    };
    let abc = PatternExpression::Sequence {
        first: Box::new(ab),
        second: Box::new(c),
    };

    assert_eq!(abc.to_string(), "e1=A{2,3} -> e2=B{1,2} -> e3=C");
}

#[test]
fn test_complex_pattern_nested_logical() {
    // (A AND B) OR C
    let a = PatternExpression::Stream {
        alias: None,
        stream_name: obj_name("A"),
        filter: None,
    };
    let b = PatternExpression::Stream {
        alias: None,
        stream_name: obj_name("B"),
        filter: None,
    };
    let c = PatternExpression::Stream {
        alias: None,
        stream_name: obj_name("C"),
        filter: None,
    };

    let and_pattern = PatternExpression::Logical {
        left: Box::new(a),
        op: PatternLogicalOp::And,
        right: Box::new(b),
    };
    let grouped = PatternExpression::Grouped {
        pattern: Box::new(and_pattern),
    };
    let or_pattern = PatternExpression::Logical {
        left: Box::new(grouped),
        op: PatternLogicalOp::Or,
        right: Box::new(c),
    };

    assert_eq!(or_pattern.to_string(), "(A AND B) OR C");
}

// ============================================================================
// Parser Tests - Pattern Clause Parsing
// ============================================================================

use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

fn parse_sql(sql: &str) -> sqlparser::ast::Statement {
    let dialect = GenericDialect {};
    let statements = Parser::parse_sql(&dialect, sql).expect("Failed to parse SQL");
    assert_eq!(statements.len(), 1, "Expected exactly one statement");
    statements.into_iter().next().unwrap()
}

fn extract_from_pattern(sql: &str) -> TableFactor {
    let stmt = parse_sql(sql);
    match stmt {
        sqlparser::ast::Statement::Query(query) => {
            match *query.body {
                sqlparser::ast::SetExpr::Select(select) => {
                    assert!(!select.from.is_empty(), "Expected FROM clause");
                    select.from.into_iter().next().unwrap().relation
                }
                _ => panic!("Expected SELECT"),
            }
        }
        _ => panic!("Expected Query"),
    }
}

#[test]
fn test_parse_from_pattern_basic() {
    let sql = "SELECT * FROM PATTERN (e1=A -> e2=B)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { mode, pattern, within, alias } => {
            assert_eq!(mode, PatternMode::Pattern);
            assert!(within.is_none());
            assert!(alias.is_none());

            // Check pattern structure
            match pattern {
                PatternExpression::Sequence { first, second } => {
                    // Verify first element is e1=A
                    match *first {
                        PatternExpression::Stream { alias, stream_name, filter } => {
                            assert_eq!(alias.unwrap().value, "e1");
                            assert_eq!(stream_name.to_string(), "A");
                            assert!(filter.is_none());
                        }
                        _ => panic!("Expected Stream for first"),
                    }
                    // Verify second element is e2=B
                    match *second {
                        PatternExpression::Stream { alias, stream_name, filter } => {
                            assert_eq!(alias.unwrap().value, "e2");
                            assert_eq!(stream_name.to_string(), "B");
                            assert!(filter.is_none());
                        }
                        _ => panic!("Expected Stream for second"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_from_sequence_basic() {
    let sql = "SELECT * FROM SEQUENCE (e1=A -> e2=B)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { mode, .. } => {
            assert_eq!(mode, PatternMode::Sequence, "Expected SEQUENCE mode");
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_stream_no_alias() {
    let sql = "SELECT * FROM PATTERN (StockStream)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Stream { alias, stream_name, filter } => {
                    assert!(alias.is_none(), "Expected no alias");
                    assert_eq!(stream_name.to_string(), "StockStream");
                    assert!(filter.is_none());
                }
                _ => panic!("Expected Stream pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_three_way_sequence() {
    let sql = "SELECT * FROM PATTERN (e1=A -> e2=B -> e3=C)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            // Pattern should be: (e1=A -> e2=B) -> e3=C
            match pattern {
                PatternExpression::Sequence { first, second } => {
                    // first should be (A -> B)
                    match *first {
                        PatternExpression::Sequence { .. } => { /* ok */ }
                        _ => panic!("Expected nested Sequence for first"),
                    }
                    // second should be C
                    match *second {
                        PatternExpression::Stream { alias, stream_name, .. } => {
                            assert_eq!(alias.unwrap().value, "e3");
                            assert_eq!(stream_name.to_string(), "C");
                        }
                        _ => panic!("Expected Stream for second"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_count_quantifier_exact() {
    let sql = "SELECT * FROM PATTERN (e1=A{3} -> e2=B)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Sequence { first, .. } => {
                    match *first {
                        PatternExpression::Count { pattern, min_count, max_count } => {
                            assert_eq!(min_count, 3);
                            assert_eq!(max_count, 3, "Exact count {{3}} should be {{3,3}}");
                            match *pattern {
                                PatternExpression::Stream { alias, stream_name, .. } => {
                                    assert_eq!(alias.unwrap().value, "e1");
                                    assert_eq!(stream_name.to_string(), "A");
                                }
                                _ => panic!("Expected Stream inside Count"),
                            }
                        }
                        _ => panic!("Expected Count pattern for first"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_count_quantifier_range() {
    let sql = "SELECT * FROM PATTERN (e1=A{2,5})";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Count { min_count, max_count, .. } => {
                    assert_eq!(min_count, 2);
                    assert_eq!(max_count, 5);
                }
                _ => panic!("Expected Count pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_logical_and() {
    let sql = "SELECT * FROM PATTERN (e1=A AND e2=B)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Logical { left, op, right } => {
                    assert_eq!(op, PatternLogicalOp::And);
                    match *left {
                        PatternExpression::Stream { alias, stream_name, .. } => {
                            assert_eq!(alias.unwrap().value, "e1");
                            assert_eq!(stream_name.to_string(), "A");
                        }
                        _ => panic!("Expected Stream for left"),
                    }
                    match *right {
                        PatternExpression::Stream { alias, stream_name, .. } => {
                            assert_eq!(alias.unwrap().value, "e2");
                            assert_eq!(stream_name.to_string(), "B");
                        }
                        _ => panic!("Expected Stream for right"),
                    }
                }
                _ => panic!("Expected Logical pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_logical_or() {
    let sql = "SELECT * FROM PATTERN (e1=A OR e2=B)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Logical { op, .. } => {
                    assert_eq!(op, PatternLogicalOp::Or);
                }
                _ => panic!("Expected Logical pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_every() {
    let sql = "SELECT * FROM PATTERN (EVERY e1=A -> e2=B)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            // The result should be: EVERY(A) -> B
            // This is because EVERY only wraps the immediately following term
            match pattern {
                PatternExpression::Sequence { first, .. } => {
                    match *first {
                        PatternExpression::Every { pattern } => {
                            match *pattern {
                                PatternExpression::Stream { alias, stream_name, .. } => {
                                    assert_eq!(alias.unwrap().value, "e1");
                                    assert_eq!(stream_name.to_string(), "A");
                                }
                                _ => panic!("Expected Stream inside Every"),
                            }
                        }
                        _ => panic!("Expected Every pattern for first"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_grouped() {
    let sql = "SELECT * FROM PATTERN ((e1=A -> e2=B) -> e3=C)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Sequence { first, second } => {
                    // first should be Grouped((A -> B))
                    match *first {
                        PatternExpression::Grouped { pattern } => {
                            match *pattern {
                                PatternExpression::Sequence { .. } => { /* ok */ }
                                _ => panic!("Expected Sequence inside Grouped"),
                            }
                        }
                        _ => panic!("Expected Grouped pattern for first"),
                    }
                    // second should be C
                    match *second {
                        PatternExpression::Stream { alias, stream_name, .. } => {
                            assert_eq!(alias.unwrap().value, "e3");
                            assert_eq!(stream_name.to_string(), "C");
                        }
                        _ => panic!("Expected Stream for second"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_within_events() {
    let sql = "SELECT * FROM PATTERN (e1=A -> e2=B) WITHIN 100 EVENTS";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { within, .. } => {
            match within {
                Some(WithinConstraint::EventCount(count)) => {
                    assert_eq!(count, 100);
                }
                _ => panic!("Expected WITHIN 100 EVENTS"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_with_alias() {
    let sql = "SELECT * FROM PATTERN (e1=A -> e2=B) AS p";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { alias, .. } => {
            assert!(alias.is_some());
            assert_eq!(alias.unwrap().name.value, "p");
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

#[test]
fn test_parse_pattern_roundtrip() {
    // Test that parsing and display produce consistent results
    let sql = "SELECT * FROM PATTERN (e1=A -> e2=B)";
    let table_factor = extract_from_pattern(sql);

    let displayed = table_factor.to_string();
    assert_eq!(displayed, "PATTERN (e1=A -> e2=B)");
}

#[test]
fn test_parse_pattern_count_quantifier_roundtrip() {
    let sql = "SELECT * FROM PATTERN (e1=A{2,5} -> e2=B)";
    let table_factor = extract_from_pattern(sql);

    let displayed = table_factor.to_string();
    assert_eq!(displayed, "PATTERN (e1=A{2,5} -> e2=B)");
}

#[test]
fn test_parse_sequence_with_count() {
    let sql = "SELECT * FROM SEQUENCE (e1=Login{3} -> e2=Logout)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { mode, pattern, .. } => {
            assert_eq!(mode, PatternMode::Sequence);
            match pattern {
                PatternExpression::Sequence { first, second } => {
                    match *first {
                        PatternExpression::Count { min_count, max_count, .. } => {
                            assert_eq!(min_count, 3);
                            assert_eq!(max_count, 3);
                        }
                        _ => panic!("Expected Count pattern"),
                    }
                    match *second {
                        PatternExpression::Stream { alias, stream_name, .. } => {
                            assert_eq!(alias.unwrap().value, "e2");
                            assert_eq!(stream_name.to_string(), "Logout");
                        }
                        _ => panic!("Expected Stream pattern"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern"),
    }
}

// ============================================================================
// Filter Condition Tests
// ============================================================================

#[test]
fn test_parse_pattern_with_filter() {
    let sql = "SELECT * FROM PATTERN (e1=StockStream[price > 100])";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Stream { alias, stream_name, filter } => {
                    assert_eq!(alias.unwrap().value, "e1");
                    assert_eq!(stream_name.to_string(), "StockStream");
                    assert!(filter.is_some(), "Expected filter condition");
                    // The filter should be: price > 100
                    let filter_str = filter.unwrap().to_string();
                    assert!(filter_str.contains("price"), "Filter should reference 'price'");
                    assert!(filter_str.contains("100"), "Filter should contain '100'");
                }
                _ => panic!("Expected Stream pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_pattern_with_filter"),
    }
}

#[test]
fn test_parse_pattern_with_complex_filter() {
    let sql = "SELECT * FROM PATTERN (e1=Trade[symbol = 'AAPL' AND quantity > 100])";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Stream { filter, .. } => {
                    assert!(filter.is_some(), "Expected filter condition");
                    let filter_str = filter.unwrap().to_string();
                    assert!(filter_str.contains("symbol"), "Filter should reference 'symbol'");
                    assert!(filter_str.contains("AAPL"), "Filter should contain 'AAPL'");
                    assert!(filter_str.contains("quantity"), "Filter should reference 'quantity'");
                }
                _ => panic!("Expected Stream pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_pattern_with_complex_filter"),
    }
}

#[test]
fn test_parse_pattern_sequence_with_filters() {
    let sql = "SELECT * FROM PATTERN (e1=A[x > 10] -> e2=B[y < 20])";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Sequence { first, second } => {
                    // Check first has filter
                    match *first {
                        PatternExpression::Stream { alias, filter, .. } => {
                            assert_eq!(alias.unwrap().value, "e1");
                            assert!(filter.is_some(), "First stream should have filter");
                        }
                        _ => panic!("Expected Stream for first"),
                    }
                    // Check second has filter
                    match *second {
                        PatternExpression::Stream { alias, filter, .. } => {
                            assert_eq!(alias.unwrap().value, "e2");
                            assert!(filter.is_some(), "Second stream should have filter");
                        }
                        _ => panic!("Expected Stream for second"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_pattern_sequence_with_filters"),
    }
}

// ============================================================================
// Time-based WITHIN Tests
// ============================================================================

#[test]
fn test_parse_pattern_within_seconds() {
    // WITHIN with time unit (converted to milliseconds internally)
    let sql = "SELECT * FROM PATTERN (e1=A -> e2=B) WITHIN 10 SECONDS";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { within, .. } => {
            match within {
                Some(WithinConstraint::Time(expr)) => {
                    let expr_str = expr.to_string();
                    // 10 seconds = 10000 milliseconds
                    assert_eq!(expr_str, "10000", "10 SECONDS should be 10000ms: {}", expr_str);
                }
                _ => panic!("Expected WITHIN Time constraint, got {:?}", within),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_pattern_within_seconds"),
    }
}

#[test]
fn test_parse_pattern_within_milliseconds() {
    // WITHIN with milliseconds (stays as-is)
    let sql = "SELECT * FROM PATTERN (e1=A -> e2=B) WITHIN 5000 MILLISECONDS";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { within, .. } => {
            match within {
                Some(WithinConstraint::Time(expr)) => {
                    let expr_str = expr.to_string();
                    assert_eq!(expr_str, "5000", "5000 MILLISECONDS should be 5000ms: {}", expr_str);
                }
                _ => panic!("Expected WITHIN Time constraint"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_pattern_within_milliseconds"),
    }
}

#[test]
fn test_parse_pattern_within_minutes() {
    // WITHIN with minutes (converted to milliseconds)
    let sql = "SELECT * FROM PATTERN (e1=A -> e2=B) WITHIN 5 MINUTES";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { within, .. } => {
            match within {
                Some(WithinConstraint::Time(expr)) => {
                    let expr_str = expr.to_string();
                    // 5 minutes = 300000 milliseconds
                    assert_eq!(expr_str, "300000", "5 MINUTES should be 300000ms: {}", expr_str);
                }
                _ => panic!("Expected WITHIN Time constraint"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_pattern_within_minutes"),
    }
}

// ============================================================================
// Cross-stream Reference Filter Tests
// ============================================================================

#[test]
fn test_parse_pattern_cross_stream_filter() {
    // e2 references e1's price
    let sql = "SELECT * FROM PATTERN (e1=Buy -> e2=Sell[price > e1.price])";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Sequence { second, .. } => {
                    match *second {
                        PatternExpression::Stream { filter, .. } => {
                            assert!(filter.is_some(), "Expected cross-stream filter");
                            let filter_str = filter.unwrap().to_string();
                            // Should reference e1.price
                            assert!(filter_str.contains("e1"), "Should reference e1: {}", filter_str);
                            assert!(filter_str.contains("price"), "Should reference price: {}", filter_str);
                        }
                        _ => panic!("Expected Stream for second"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_pattern_cross_stream_filter"),
    }
}

// ============================================================================
// Complex Pattern Tests
// ============================================================================

#[test]
fn test_parse_complex_pattern_with_all_features() {
    // Pattern with count, filter, and WITHIN
    let sql = "SELECT * FROM PATTERN (e1=A{2,3}[value > 0] -> e2=B) WITHIN 100 EVENTS";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { mode, pattern, within, .. } => {
            assert_eq!(mode, PatternMode::Pattern);

            // Check WITHIN
            match within {
                Some(WithinConstraint::EventCount(count)) => {
                    assert_eq!(count, 100);
                }
                _ => panic!("Expected WITHIN 100 EVENTS"),
            }

            // Check pattern structure
            match pattern {
                PatternExpression::Sequence { first, .. } => {
                    match *first {
                        PatternExpression::Count { pattern, min_count, max_count } => {
                            assert_eq!(min_count, 2);
                            assert_eq!(max_count, 3);
                            match *pattern {
                                PatternExpression::Stream { filter, .. } => {
                                    assert!(filter.is_some(), "Expected filter on counted stream");
                                }
                                _ => panic!("Expected Stream inside Count"),
                            }
                        }
                        _ => panic!("Expected Count pattern"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_complex_pattern_with_all_features"),
    }
}

#[test]
fn test_parse_pattern_every_with_filter() {
    let sql = "SELECT * FROM PATTERN (EVERY e1=Login[userId IS NOT NULL] -> e2=Logout)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Sequence { first, .. } => {
                    match *first {
                        PatternExpression::Every { pattern } => {
                            match *pattern {
                                PatternExpression::Stream { alias, filter, .. } => {
                                    assert_eq!(alias.unwrap().value, "e1");
                                    assert!(filter.is_some(), "Expected filter in EVERY pattern");
                                }
                                _ => panic!("Expected Stream inside Every"),
                            }
                        }
                        _ => panic!("Expected Every pattern"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_pattern_every_with_filter"),
    }
}

#[test]
fn test_parse_pattern_logical_with_sequence() {
    // (A AND B) -> C pattern
    let sql = "SELECT * FROM PATTERN ((e1=A AND e2=B) -> e3=C)";
    let table_factor = extract_from_pattern(sql);

    match table_factor {
        TableFactor::Pattern { pattern, .. } => {
            match pattern {
                PatternExpression::Sequence { first, second } => {
                    // first should be Grouped(A AND B)
                    match *first {
                        PatternExpression::Grouped { pattern } => {
                            match *pattern {
                                PatternExpression::Logical { op, .. } => {
                                    assert_eq!(op, PatternLogicalOp::And);
                                }
                                _ => panic!("Expected Logical inside Grouped"),
                            }
                        }
                        _ => panic!("Expected Grouped pattern"),
                    }
                    // second should be C
                    match *second {
                        PatternExpression::Stream { alias, .. } => {
                            assert_eq!(alias.unwrap().value, "e3");
                        }
                        _ => panic!("Expected Stream for second"),
                    }
                }
                _ => panic!("Expected Sequence pattern"),
            }
        }
        _ => panic!("Expected TableFactor::Pattern in test_parse_pattern_logical_with_sequence"),
    }
}

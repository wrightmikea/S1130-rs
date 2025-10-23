//! Parser for IBM 1130 Assembly Language
//!
//! Parses tokens into structured assembly lines.

use crate::error::AssemblerError;

/// Result type for assembler operations
pub type Result<T> = std::result::Result<T, AssemblerError>;

/// Parsed assembly line
#[derive(Debug, Clone)]
pub struct ParsedLine {
    /// Optional label
    pub label: Option<String>,

    /// Operation (instruction or pseudo-op)
    pub operation: Operation,

    /// Optional operand
    pub operand: Option<String>,
}

/// Operation type
#[derive(Debug, Clone)]
pub enum Operation {
    /// Machine instruction
    Instruction(String),

    /// Pseudo-operation
    PseudoOp(String),

    /// No operation (comment or empty line)
    None,
}

/// Parse source code into lines
pub fn parse_source(source: &str) -> Result<Vec<ParsedLine>> {
    let mut lines = Vec::new();

    for (line_num, line_text) in source.lines().enumerate() {
        let parsed = parse_line(line_text, line_num + 1)?;
        if !matches!(parsed.operation, Operation::None) || parsed.label.is_some() {
            lines.push(parsed);
        }
    }

    Ok(lines)
}

/// Parse a single line
fn parse_line(line: &str, line_num: usize) -> Result<ParsedLine> {
    // Don't trim initially - we need to check for leading whitespace
    let original_line = line;

    // Skip empty lines
    if original_line.trim().is_empty() {
        return Ok(ParsedLine {
            label: None,
            operation: Operation::None,
            operand: None,
        });
    }

    // Check for full-line comment (starts with *)
    if original_line.trim_start().starts_with('*') {
        return Ok(ParsedLine {
            label: None,
            operation: Operation::None,
            operand: None,
        });
    }

    // Strip inline comments (everything after first '*' that's not at position 0)
    let line_without_comment = if let Some(comment_pos) = original_line.find('*') {
        if comment_pos > 0 {
            &original_line[..comment_pos]
        } else {
            original_line
        }
    } else {
        original_line
    };

    // Check if line starts with whitespace to determine if there's a label
    let has_leading_whitespace = line_without_comment
        .chars()
        .next()
        .map(|c| c.is_whitespace())
        .unwrap_or(false);

    // Now split into tokens (this will trim each token)
    let parts: Vec<&str> = line_without_comment.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(ParsedLine {
            label: None,
            operation: Operation::None,
            operand: None,
        });
    }

    let mut label = None;
    let mut operation = Operation::None;
    let mut operand = None;

    if has_leading_whitespace {
        // No label - line starts with whitespace
        // First token is operation
        let first = parts[0];

        if is_instruction(first) {
            operation = Operation::Instruction(first.to_uppercase());
        } else if is_pseudo_op(first) {
            operation = Operation::PseudoOp(first.to_uppercase());
        } else {
            return Err(AssemblerError::SyntaxError {
                line: line_num,
                message: format!("Expected instruction or pseudo-op, got: {}", first),
            });
        }

        // Rest is operand
        if parts.len() > 1 {
            operand = Some(parts[1..].join(" "));
        }
    } else {
        // Line starts with non-whitespace - first token is label
        label = Some(parts[0].to_string());

        if parts.len() > 1 {
            let second = parts[1];

            if is_instruction(second) {
                operation = Operation::Instruction(second.to_uppercase());

                if parts.len() > 2 {
                    operand = Some(parts[2..].join(" "));
                }
            } else if is_pseudo_op(second) {
                operation = Operation::PseudoOp(second.to_uppercase());

                if parts.len() > 2 {
                    operand = Some(parts[2..].join(" "));
                }
            } else {
                return Err(AssemblerError::SyntaxError {
                    line: line_num,
                    message: format!("Expected instruction or pseudo-op, got: {}", second),
                });
            }
        }
        // If only one token (label only), operation stays None
    }

    Ok(ParsedLine {
        label,
        operation,
        operand,
    })
}

/// Check if string is a valid instruction
fn is_instruction(s: &str) -> bool {
    matches!(
        s.to_uppercase().as_str(),
        "LD" | "LDD"
            | "STO"
            | "STD"
            | "A"
            | "AD"
            | "S"
            | "SD"
            | "M"
            | "D"
            | "AND"
            | "OR"
            | "EOR"
            | "SLA"
            | "SLCA"
            | "SRA"
            | "SRT"
            | "BSI"
            | "BSC"
            | "BC"
            | "LDX"
            | "STX"
            | "MDX"
            | "WAIT"
            | "LDS"
            | "STS"
            | "XIO"
            | "SDS"
    )
}

/// Check if string is a pseudo-op
fn is_pseudo_op(s: &str) -> bool {
    matches!(
        s.to_uppercase().as_str(),
        "ORG" | "DC" | "BSS" | "END" | "EQU"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_instruction() {
        let line = parse_line("    LD 100", 1).unwrap();
        assert!(line.label.is_none());
        assert!(matches!(line.operation, Operation::Instruction(_)));
        assert_eq!(line.operand, Some("100".to_string()));
    }

    #[test]
    fn test_parse_with_label() {
        let line = parse_line("START LD 100", 1).unwrap();
        assert_eq!(line.label, Some("START".to_string()));
        assert!(matches!(line.operation, Operation::Instruction(_)));
        assert_eq!(line.operand, Some("100".to_string()));
    }

    #[test]
    fn test_parse_pseudo_op() {
        let line = parse_line("    ORG 0x100", 1).unwrap();
        assert!(line.label.is_none());
        assert!(matches!(line.operation, Operation::PseudoOp(_)));
        assert_eq!(line.operand, Some("0x100".to_string()));
    }

    #[test]
    fn test_parse_label_only() {
        let line = parse_line("LABEL", 1).unwrap();
        assert_eq!(line.label, Some("LABEL".to_string()));
        assert!(matches!(line.operation, Operation::None));
    }

    #[test]
    fn test_parse_comment() {
        let line = parse_line("* This is a comment", 1).unwrap();
        assert!(matches!(line.operation, Operation::None));
    }

    #[test]
    fn test_parse_empty_line() {
        let line = parse_line("", 1).unwrap();
        assert!(matches!(line.operation, Operation::None));
    }

    #[test]
    fn test_parse_indirect_operand() {
        let line = parse_line("    LD /100", 1).unwrap();
        assert_eq!(line.operand, Some("/100".to_string()));
    }

    #[test]
    fn test_parse_indexed_operand() {
        let line = parse_line("    LD 100,1", 1).unwrap();
        assert_eq!(line.operand, Some("100,1".to_string()));
    }
}

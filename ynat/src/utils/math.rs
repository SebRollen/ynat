/// Evaluate a simple math expression containing +, -, *, /, and parentheses.
/// Returns the result as a formatted string with 2 decimal places, or None if invalid.
///
/// Examples:
/// - "10+5" -> Some("15.00")
/// - "100-20*2" -> Some("60.00")
/// - "(100-20)*2" -> Some("160.00")
/// - "-50" -> Some("-50.00")
pub fn evaluate_expression(expr: &str) -> Option<String> {
    let expr = expr.trim();
    if expr.is_empty() {
        return None;
    }

    // If it's already a simple number, just parse and format it
    if let Ok(num) = expr.parse::<f64>() {
        return Some(format!("{:.2}", num));
    }

    // Check if the expression contains any operators
    if !expr
        .chars()
        .any(|c| matches!(c, '+' | '*' | '/' | '(' | ')') || (c == '-' && expr.len() > 1))
    {
        return None;
    }

    let mut parser = ExprParser::new(expr);
    parser
        .parse_expression()
        .map(|result| format!("{:.2}", result))
}

/// Simple recursive descent parser for math expressions
struct ExprParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> ExprParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Parse a complete expression (handles + and - at lowest precedence)
    fn parse_expression(&mut self) -> Option<f64> {
        self.skip_whitespace();
        let mut left = self.parse_term()?;

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('+') => {
                    self.advance();
                    let right = self.parse_term()?;
                    left += right;
                }
                Some('-') => {
                    self.advance();
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }

        self.skip_whitespace();
        // Ensure we consumed the entire input
        if self.pos == self.input.len() {
            Some(left)
        } else {
            None
        }
    }

    /// Parse a term (handles * and / at higher precedence)
    fn parse_term(&mut self) -> Option<f64> {
        self.skip_whitespace();
        let mut left = self.parse_factor()?;

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('*') => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left *= right;
                }
                Some('/') => {
                    self.advance();
                    let right = self.parse_factor()?;
                    if right == 0.0 {
                        return None; // Division by zero
                    }
                    left /= right;
                }
                _ => break,
            }
        }

        Some(left)
    }

    /// Parse a factor (number, unary +/-, or parenthesized expression)
    fn parse_factor(&mut self) -> Option<f64> {
        self.skip_whitespace();

        match self.peek() {
            Some('(') => {
                self.advance();
                let inner = self.parse_expression_inner()?;
                self.skip_whitespace();
                if self.peek() == Some(')') {
                    self.advance();
                    Some(inner)
                } else {
                    None // Missing closing paren
                }
            }
            Some('-') => {
                self.advance();
                let factor = self.parse_factor()?;
                Some(-factor)
            }
            Some('+') => {
                self.advance();
                self.parse_factor() // unary + is a no-op
            }
            Some(c) if c.is_ascii_digit() || c == '.' => self.parse_number(),
            _ => None,
        }
    }

    /// Parse expression inside parentheses (doesn't check for end of input)
    fn parse_expression_inner(&mut self) -> Option<f64> {
        self.skip_whitespace();
        let mut left = self.parse_term()?;

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('+') => {
                    self.advance();
                    let right = self.parse_term()?;
                    left += right;
                }
                Some('-') => {
                    self.advance();
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }

        Some(left)
    }

    /// Parse a number (integer or decimal)
    fn parse_number(&mut self) -> Option<f64> {
        let start = self.pos;

        // Consume digits and at most one decimal point
        let mut has_decimal = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else if c == '.' && !has_decimal {
                has_decimal = true;
                self.advance();
            } else {
                break;
            }
        }

        if self.pos == start {
            return None;
        }

        self.input[start..self.pos].parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_expression_simple_numbers() {
        assert_eq!(evaluate_expression("50"), Some("50.00".to_string()));
        assert_eq!(evaluate_expression("-50"), Some("-50.00".to_string()));
        assert_eq!(evaluate_expression("50.5"), Some("50.50".to_string()));
        assert_eq!(evaluate_expression("-50.25"), Some("-50.25".to_string()));
    }

    #[test]
    fn evaluate_expression_addition_subtraction() {
        assert_eq!(evaluate_expression("10+5"), Some("15.00".to_string()));
        assert_eq!(evaluate_expression("100-20"), Some("80.00".to_string()));
        assert_eq!(evaluate_expression("10+5-3"), Some("12.00".to_string()));
        assert_eq!(evaluate_expression("10 + 5"), Some("15.00".to_string())); // with spaces
    }

    #[test]
    fn evaluate_expression_multiplication_division() {
        assert_eq!(evaluate_expression("10*5"), Some("50.00".to_string()));
        assert_eq!(evaluate_expression("100/4"), Some("25.00".to_string()));
        assert_eq!(evaluate_expression("10*5/2"), Some("25.00".to_string()));
    }

    #[test]
    fn evaluate_expression_operator_precedence() {
        // Multiplication before addition
        assert_eq!(evaluate_expression("10+5*2"), Some("20.00".to_string()));
        assert_eq!(evaluate_expression("100-20*2"), Some("60.00".to_string()));
        assert_eq!(evaluate_expression("2*3+4*5"), Some("26.00".to_string()));
    }

    #[test]
    fn evaluate_expression_parentheses() {
        assert_eq!(evaluate_expression("(10+5)*2"), Some("30.00".to_string()));
        assert_eq!(
            evaluate_expression("(100-20)*2"),
            Some("160.00".to_string())
        );
        assert_eq!(evaluate_expression("2*(3+4)"), Some("14.00".to_string()));
        assert_eq!(
            evaluate_expression("(10+5)*(2+3)"),
            Some("75.00".to_string())
        );
    }

    #[test]
    fn evaluate_expression_negative_numbers() {
        assert_eq!(evaluate_expression("-50+10"), Some("-40.00".to_string()));
        assert_eq!(evaluate_expression("10+-5"), Some("5.00".to_string()));
        assert_eq!(evaluate_expression("10*-2"), Some("-20.00".to_string()));
        assert_eq!(evaluate_expression("(-10)*5"), Some("-50.00".to_string()));
    }

    #[test]
    fn evaluate_expression_unary_plus() {
        assert_eq!(evaluate_expression("+20"), Some("20.00".to_string()));
        assert_eq!(evaluate_expression("+50.5"), Some("50.50".to_string()));
        assert_eq!(evaluate_expression("10++5"), Some("15.00".to_string()));
        assert_eq!(evaluate_expression("(+10)*2"), Some("20.00".to_string()));
    }

    #[test]
    fn evaluate_expression_invalid() {
        assert_eq!(evaluate_expression(""), None);
        assert_eq!(evaluate_expression("abc"), None);
        assert_eq!(evaluate_expression("10+"), None);
        assert_eq!(evaluate_expression("(10+5"), None); // missing closing paren
        assert_eq!(evaluate_expression("10/0"), None); // division by zero
    }
}

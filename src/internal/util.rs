use internal::error::ReployError;
use std::path::PathBuf;

pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

pub fn ssh_key() -> PathBuf {
    home_dir()
        .map(|d| d.join(".ssh").join("id_rsa"))
        .unwrap_or(PathBuf::new())
}

pub fn is_expression(expr: &str) -> bool {
    expr.contains('*')
        || expr.contains('/')
        || expr.contains('+')
        || expr.contains('-')
        || expr.contains('<')
        || expr.contains('>')
        || expr.contains("==")
        || expr.contains("!=")
        || expr.contains(">=")
        || expr.contains("<=")
}

pub fn evaluate_expression(expr: String) -> Result<String, ReployError> {
    let expr = expr.trim_start_matches("{{").trim_end_matches("}}");
    // If not an expression, return as-is
    if !is_expression(expr) {
        return Ok(expr.to_string());
    }

    // Handle multiplication and division first (higher precedence)
    if let Some(pos) = expr.find('*') {
        let left = expr[..pos].trim().to_string();
        let right = expr[pos + 1..].trim().to_string();
        let left_num = left
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid number: {}", left)))?;
        let right_num = right
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid number: {}", right)))?;
        return Ok((left_num * right_num).to_string());
    }

    if let Some(pos) = expr.find('/') {
        let left = expr[..pos].trim().to_string();
        let right = expr[pos + 1..].trim().to_string();
        let left_num = left
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid number: {}", left)))?;
        let right_num = right
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid number: {}", right)))?;
        if right_num == 0 {
            return Err(ReployError::Runtime("Division by zero".to_string()));
        }
        return Ok((left_num / right_num).to_string());
    }

    // Handle addition and subtraction
    if let Some(pos) = expr.find('+') {
        let left = expr[..pos].trim().to_string();
        let right = expr[pos + 1..].trim().to_string();
        let left_num = left
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid number: {}", left)))?;
        let right_num = right
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid number: {}", right)))?;
        return Ok((left_num + right_num).to_string());
    }

    if let Some(pos) = expr.find('-') {
        let left = expr[..pos].trim().to_string();
        let right = expr[pos + 1..].trim().to_string();
        let left_num = left
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid number: {}", left)))?;
        let right_num = right
            .parse::<i32>()
            .map_err(|_| ReployError::Runtime(format!("Invalid number: {}", right)))?;
        return Ok((left_num - right_num).to_string());
    }

    // Handle comparison operators
    if expr.contains("<") {
        let parts: Vec<&str> = expr.split('<').collect();
        if parts.len() == 2 {
            let left = parts[0].trim().to_string();
            let right = parts[1].trim().to_string();
            let left_num = left.parse::<i32>().unwrap_or(0);
            let right_num = right.parse::<i32>().unwrap_or(0);
            return Ok((left_num < right_num).to_string());
        }
    }

    if expr.contains(">") {
        let parts: Vec<&str> = expr.split('>').collect();
        if parts.len() == 2 {
            let left = parts[0].trim().to_string();
            let right = parts[1].trim().to_string();
            let left_num = left.parse::<i32>().unwrap_or(0);
            let right_num = right.parse::<i32>().unwrap_or(0);
            return Ok((left_num > right_num).to_string());
        }
    }

    if expr.contains("==") {
        let parts: Vec<&str> = expr.split("==").collect();
        if parts.len() == 2 {
            let left = parts[0].trim().to_string();
            let right = parts[1].trim().to_string();
            return Ok((left == right).to_string());
        }
    }

    if expr.contains("!=") {
        let parts: Vec<&str> = expr.split("!=").collect();
        if parts.len() == 2 {
            let left = parts[0].trim().to_string();
            let right = parts[1].trim().to_string();
            return Ok((left != right).to_string());
        }
    }

    if expr.contains(">=") {
        let parts: Vec<&str> = expr.split(">=").collect();
        if parts.len() == 2 {
            let left = parts[0].trim().to_string();
            let right = parts[1].trim().to_string();
            let left_num = left.parse::<i32>().unwrap_or(0);
            let right_num = right.parse::<i32>().unwrap_or(0);
            return Ok((left_num >= right_num).to_string());
        }
    }

    if expr.contains("<=") {
        let parts: Vec<&str> = expr.split(">=").collect();
        if parts.len() == 2 {
            let left = parts[0].trim().to_string();
            let right = parts[1].trim().to_string();
            let left_num = left.parse::<i32>().unwrap_or(0);
            let right_num = right.parse::<i32>().unwrap_or(0);
            return Ok((left_num <= right_num).to_string());
        }
    }

    return Err(ReployError::Runtime(format!(
        "Invalid expression: {}",
        expr
    )));
}

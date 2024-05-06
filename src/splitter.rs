pub fn split(command: String) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::from("");

    let mut in_quotes = false;
    let mut quote_char = '\0';

    for character in command.chars() {
        if (character == '\'' || character == '\"') && (!in_quotes || character == quote_char) {
            in_quotes = !in_quotes;
            quote_char = character;
            current.push(character);
        } else if character == ' ' && !in_quotes {
            if !current.is_empty() {
                result.push(current.clone());
                current.clear();
            }
        } else {
            current.push(character);
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_test() {
        let r = split(String::from("ls -la -h"));
        assert_eq!(r, vec!["ls", "-la", "-h"]);

        let r = split(String::from("ls -la --env=\"VAR VAR\" -h"));
        assert_eq!(r, vec!["ls", "-la", "--env=\"VAR VAR\"", "-h"]);
    }
}

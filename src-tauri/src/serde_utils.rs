use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;

/// 将 snake_case 的字段名转换为 camelCase
pub fn snake_to_camel(snake: &str) -> String {
    let parts: Vec<&str> = snake.split('_').collect();
    if parts.len() == 1 {
        return snake.to_string();
    }

    parts[0].to_string() + &parts[1..].iter()
        .map(|s| {
            let mut chars = s.chars();
            if let Some(first) = chars.next() {
                first.to_uppercase().collect::<String>() + &chars.collect::<String>()
            } else {
                String::new()
            }
        })
        .collect::<String>()
}

/// 将 camelCase 的字段名转换为 snake_case
pub fn camel_to_snake(camel: &str) -> String {
    let mut result = String::new();
    let mut prev_upper = false;

    for (i, c) in camel.chars().enumerate() {
        if c.is_uppercase() && i > 0 && !prev_upper {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
        prev_upper = c.is_uppercase();
    }

    result
}

/// 自定义序列化器 - 将 struct 序列化为 snake_case JSON
pub fn serialize_to_snake<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let json = serde_json::to_value(value).map_err(serde::ser::Error::custom)?;
    let snake_json = convert_keys_to_snake(&json);
    snake_json.serialize(serializer)
}

/// 自定义反序列化器 - 将 snake_case JSON 反序列化为 struct
pub fn deserialize_from_snake<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let json = Value::deserialize(deserializer)?;
    let snake_json = convert_keys_to_snake(&json);
    T::deserialize(snake_json).map_err(|e| serde::de::Error::custom(e.to_string()))
}

/// 递归转换 JSON 对象中的所有键为 snake_case
pub fn convert_keys_to_snake(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                let snake_key = camel_to_snake(key);
                new_map.insert(snake_key, convert_keys_to_snake(val));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(convert_keys_to_snake).collect())
        }
        _ => value.clone(),
    }
}

/// 递归转换 JSON 对象中的所有键为 camelCase
pub fn convert_keys_to_camel(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                let camel_key = snake_to_camel(key);
                new_map.insert(camel_key, convert_keys_to_camel(val));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(convert_keys_to_camel).collect())
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("hello"), "hello");
        assert_eq!(snake_to_camel("hello_world"), "helloWorld");
        assert_eq!(snake_to_camel("connection_id"), "connectionId");
        assert_eq!(snake_to_camel("local_port"), "localPort");
        assert_eq!(snake_to_camel("auto_reconnect"), "autoReconnect");
    }

    #[test]
    fn test_camel_to_snake() {
        assert_eq!(camel_to_snake("hello"), "hello");
        assert_eq!(camel_to_snake("helloWorld"), "hello_world");
        assert_eq!(camel_to_snake("connectionId"), "connection_id");
        assert_eq!(camel_to_snake("localPort"), "local_port");
        assert_eq!(camel_to_snake("autoReconnect"), "auto_reconnect");
    }

    #[test]
    fn test_json_conversion() {
        let snake_json = serde_json::json!({
            "connection_id": "123",
            "local_port": 8080,
            "remote_host": "localhost",
            "auto_reconnect": true
        });

        let camel_json = convert_keys_to_camel(&snake_json);
        assert_eq!(camel_json["connectionId"], "123");
        assert_eq!(camel_json["localPort"], 8080);
        assert_eq!(camel_json["autoReconnect"], true);

        let back_to_snake = convert_keys_to_snake(&camel_json);
        assert_eq!(back_to_snake, snake_json);
    }
}
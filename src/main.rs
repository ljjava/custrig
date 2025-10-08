// use reqwest;
// use serde_json::{json, Value};
// use rig::{
//     parallel,
//     pipeline::{self, Op, passthrough},
// };
 
// // 自定义 Qwen 客户端
// struct QwenClient {
//     http_client: reqwest::Client,
//     api_key: String,
//     base_url: String,
// }
 
// impl QwenClient {
//     fn new(api_key: &str) -> Self {
//         Self {
//             http_client: reqwest::Client::new(),
//             api_key: api_key.to_string(),
//             base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
//         }
//     }
 
//     async fn prompt(&self, model: &str, prompt: &str, system: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
//         let mut messages = Vec::new();
        
//         if let Some(sys) = system {
//             messages.push(json!({
//                 "role": "system",
//                 "content": sys
//             }));
//         }
        
//         messages.push(json!({
//             "role": "user", 
//             "content": prompt  // 移除 /no_think，用参数控制
//         }));
 
//         let request_body = json!({
//             "model": model,
//             "messages": messages,
//             "temperature": 0.7,
//             "max_tokens": 150,
//             "enable_thinking": false  // 关键：添加这个参数
//         });
 
//         println!("发送请求: {}", serde_json::to_string_pretty(&request_body)?);
 
//         let response = self.http_client
//             .post(&format!("{}/chat/completions", self.base_url))
//             .header("Authorization", format!("Bearer {}", self.api_key))
//             .header("Content-Type", "application/json")
//             .json(&request_body)
//             .send()
//             .await?;
 
//         println!("响应状态码: {}", response.status());
        
//         let response_text = response.text().await?;
//         println!("原始响应: {}", response_text);
 
//         let result: Value = serde_json::from_str(&response_text)?;
 
//         // 更安全的解析方式
//         let content = result
//             .get("choices")
//             .and_then(|choices| choices.get(0))
//             .and_then(|choice| choice.get("message"))
//             .and_then(|message| message.get("content"))
//             .and_then(|content| content.as_str())
//             .unwrap_or("无法解析响应内容")
//             .to_string();
        
//         Ok(content)
//     }
// }
 
// // 实现 Op trait 用于 pipeline
// struct QwenNode {
//     client: QwenClient,
//     model: String,
//     system_prompt: Option<String>,
// }
 
// impl Op for QwenNode {
//     type Input = String;
//     type Output = Result<String, String>;
 
//     async fn call(&self, input: Self::Input) -> Self::Output {
//         match self.client.prompt(&self.model, &input, self.system_prompt.as_deref()).await {
//             Ok(response) => Ok(response),
//             Err(e) => Err(format!("调用失败: {}", e)),
//         }
//     }
// }
 
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let client = QwenClient::new("sk-24cc6b376bed4063b983c8feb3927d28");
 
//     // // 方案1：简单调用
//     // println!("=== 简单调用 ===");
//     // let response1 = client.prompt("qwen3-14b", "你好", Some("简洁回答，80-120字符")).await?;
//     // println!("响应1: {}", response1);
 
//     // let response2 = client.prompt("qwen3-14b", "副热带高压为什么会北移", Some("详细回答")).await?;
//     // println!("响应2: {}", response2);
 
//     // 方案2：Pipeline 并行处理
//     println!("\n=== Pipeline 并行处理 ===");
//     let node1 = QwenNode {
//         client: client.clone(),
//         model: "qwen3-14b".to_string(),
//         system_prompt: Some("简洁回答，80-120字符，无emoji".to_string()),
//     };
 
//     let node2 = QwenNode {
//         client: client.clone(),
//         model: "qwen3-14b".to_string(),
//         system_prompt: Some("简洁回答，80-120字符，无emoji".to_string()),
//     };
 
//     let node3 = QwenNode {
//         client: client.clone(),
//         model: "qwen3-14b".to_string(),
//         system_prompt: Some("简洁回答，80-120字符，无emoji".to_string()),
//     };
 
//     let chain = pipeline::new()
//         .chain(parallel!(
//             passthrough(),
//             node1,
//             node2,
//             node3
//         ))
//         .map(|(original, r1, r2, r3)| {
//             format!(
//                 "问题: {}\n简洁: {}\n详细: {}\n通俗: {}",
//                 original,
//                 r1.unwrap_or_else(|e| e),
//                 r2.unwrap_or_else(|e| e),
//                 r3.unwrap_or_else(|e| e),
//             )
//         });
 
//     let result = chain.call("副热带高压为什么会北移".to_string()).await;
//     println!("{}", result);
//     Ok(())
// }
 
// // 为了支持 clone
// impl Clone for QwenClient {
//     fn clone(&self) -> Self {
//         Self {
//             http_client: reqwest::Client::new(),
//             api_key: self.api_key.clone(),
//             base_url: self.base_url.clone(),
//         }
//     }
// }
use reqwest;
use serde_json::{json, Value};
use rig::{
    parallel,
    pipeline::{self, Op, passthrough},
};
 
struct QwenNode {
    client: reqwest::Client,
    api_key: String,
    system_prompt: String,
}
impl Op for QwenNode {
    type Input = String;
    type Output = String;
 
    async fn call(&self, input: Self::Input) -> Self::Output {
        let messages = vec![
            json!({"role": "system", "content": self.system_prompt}),
            json!({"role": "user", "content": input})
        ];
        let response = self.client
            .post("https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "qwen3-14b",
                "messages": messages,
                "enable_thinking": false
            }))
            .send()
            .await
            .unwrap();
 
        let result: Value = response.json().await.unwrap();
        result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("解析失败")
            .to_string()
    }
}
 
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "sk-24cc6b376bed4063b983c8feb3927d28";
    let client = reqwest::Client::new();
 
    let node1 = QwenNode {
        client: client.clone(),
        api_key: api_key.to_string(),
        system_prompt: "简洁回答，80-120字符，无emoji符号,压缩成一行文本".to_string(),
    };
 
    let node2 = QwenNode {
        client: client.clone(),
        api_key: api_key.to_string(),
        system_prompt: "简洁回答，80-120字符，无emoji符号,压缩成一行文本".to_string(),
    };
 
    let node3 = QwenNode {
        client: client.clone(),
        api_key: api_key.to_string(),
        system_prompt: "简洁回答，80-120字符，无emoji符号,压缩成一行文本".to_string(),
    };
 
    let chain = pipeline::new()
        .chain(parallel!(
            passthrough(),
            node1,
            node2,
            node3
        ))
        .map(|(original, r1, r2, r3)| {
            format!("问题: {}\nresp_1: {}\nresp_2: {}\nresp_3: {}", original, r1, r2, r3)
        });
 
    let result = chain.call("副热带高压为什么会北移".to_string()).await;
    println!("{}", result);
 
    Ok(())
}
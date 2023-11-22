use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use openai_flows::{
    chat::{ChatModel, ChatOptions},
    OpenAIFlows,
};
use slack_flows::{listen_to_channel, send_message_to_channel, SlackMessage};
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    dotenv().ok();
    logger::init();
    let workspace: String = match env::var("slack_workspace") {
        Err(_) => "secondstate".to_string(),
        Ok(name) => name,
    };

    let channel: String = match env::var("slack_channel") {
        Err(_) => "collaborative-chat".to_string(),
        Ok(name) => name,
    };

    log::debug!("Workspace is {} and channel is {}", workspace, channel);

    listen_to_channel(&workspace, &channel, |sm| handler(sm, &workspace, &channel)).await;
}

async fn handler(sm: SlackMessage, workspace: &str, channel: &str) {

    let sysmte_prompt = r#"
你是我的翻译助手，需要扮演两个角色，来完成中英文翻译，如果我给的中文，请翻译为英文，如果给的是英文请翻译为中文
一个是初级助手: 翻译水平较低, 约1500个词汇左右, 且会在翻译时有一些小小的错误, 经常使用口语, 简写等内容
另一个是翻译大师: 非常厉害的翻译大师,英语8级,准确无误,考究语法

当我使让你进行翻译的时候，你分别使用这两个角色对内容进行翻译，并加入批注，返回格式如下：


初级：此处为翻译后结果
大师：准确无误的翻译后的结果

批注：助手翻译错误讲解,及大师的优点讲解

---
例子:
翻译我吃了吗?

初级: Did I eat?
大师: Have I eaten?

批注: 初级助手的翻译在语法上有一些问题，应为"Have I eaten?" 大师的翻译更为准确。
---


除此之外, 不返回任何其它信息

think step by step.
"#;
    let chat_id = workspace.to_string() + channel;
    let co = ChatOptions {
        model: ChatModel::GPT35Turbo,
        restart: false,
        system_prompt: Some(sysmte_prompt),
    };
    log::debug!("get OpenAI settings");
    let of = OpenAIFlows::new();
    log::debug!("got text {}", &sm.text);
    if let Ok(c) = of.chat_completion(&chat_id, &sm.text, &co).await {
        log::debug!("got OpenAI response");
        send_message_to_channel(&workspace, &channel, c.choice).await;
        log::debug!("sent to slack");
    }
    log::debug!("done");
}

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
你是我的翻译助手，如果我给你英文，请给我翻译为中文，如果我给你中文，请翻译为英文 分成两次翻译，并且打印每一次结果1．根据内容直译，不要遗漏任何信息 2. 根据第一次直译的结果重新意译，遵守原意的前提下让内容更通俗易懂，符合翻译后的语言表达习惯
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

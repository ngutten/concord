use tracing::warn;

use crate::engine::chat_engine::ChatEngine;
use crate::engine::events::SessionId;

use super::formatter;
use super::parser::IrcMessage;

/// Process a single IRC command from a registered (authenticated) client.
/// Returns a list of lines to send back to the client.
pub fn handle_command(
    engine: &ChatEngine,
    session_id: SessionId,
    nick: &str,
    msg: &IrcMessage,
) -> Vec<String> {
    match msg.command.as_str() {
        "JOIN" => handle_join(engine, session_id, nick, msg),
        "PART" => handle_part(engine, session_id, nick, msg),
        "PRIVMSG" => handle_privmsg(engine, session_id, nick, msg),
        "TOPIC" => handle_topic(engine, session_id, nick, msg),
        "NAMES" => handle_names(engine, nick, msg),
        "LIST" => handle_list(engine, nick),
        "WHO" => handle_who(engine, nick, msg),
        "WHOIS" => handle_whois(engine, nick, msg),
        "QUIT" => vec![], // Handled at connection level
        "PING" => {
            let token = msg.params.first().map(|s| s.as_str()).unwrap_or("concord");
            vec![formatter::pong(token)]
        }
        "PONG" => vec![], // Just acknowledge, no response needed
        "NICK" | "USER" | "PASS" => {
            vec![formatter::err_alreadyregistered(nick)]
        }
        // CAP, MODE — common client sends these, just ignore or give minimal response
        "CAP" => {
            // Many modern clients send CAP LS. We don't support capabilities yet.
            // Respond with CAP * LS : (empty capability list)
            if msg.params.first().map(|s| s.as_str()) == Some("LS") {
                vec![format!(
                    ":{} CAP * LS :",
                    formatter::server_name()
                )]
            } else if msg.params.first().map(|s| s.as_str()) == Some("END") {
                vec![] // Acknowledged
            } else {
                vec![]
            }
        }
        "MODE" => {
            // Minimal MODE support — just return the channel name
            // Real mode support will come in a later phase
            if let Some(target) = msg.params.first() {
                if target.starts_with('#') {
                    vec![format!(
                        ":{} 324 {} {} +",
                        formatter::server_name(),
                        nick,
                        target
                    )]
                } else {
                    vec![format!(
                        ":{} 221 {} +",
                        formatter::server_name(),
                        nick
                    )]
                }
            } else {
                vec![formatter::err_needmoreparams(nick, "MODE")]
            }
        }
        "USERHOST" | "ISON" => {
            // Some clients send these — return empty results
            vec![]
        }
        _ => {
            warn!(command = %msg.command, "unknown IRC command");
            vec![formatter::err_unknowncommand(nick, &msg.command)]
        }
    }
}

fn handle_join(
    engine: &ChatEngine,
    session_id: SessionId,
    nick: &str,
    msg: &IrcMessage,
) -> Vec<String> {
    let Some(channels_param) = msg.params.first() else {
        return vec![formatter::err_needmoreparams(nick, "JOIN")];
    };

    let mut replies = Vec::new();

    // JOIN supports comma-separated channels: JOIN #a,#b,#c
    for channel in channels_param.split(',') {
        let channel = channel.trim();
        if channel.is_empty() {
            continue;
        }

        match engine.join_channel(session_id, channel) {
            Ok(()) => {
                // The engine already sent Join/Topic/Names events to the session's
                // mpsc channel, but for IRC we need to format them ourselves.
                // The connection handler's event loop will convert ChatEvents to
                // IRC lines. So we don't need to return anything extra here —
                // the events will arrive via the mpsc receiver.
            }
            Err(e) => {
                warn!(error = %e, %channel, "JOIN failed");
                replies.push(formatter::err_nosuchchannel(nick, channel));
            }
        }
    }

    replies
}

fn handle_part(
    engine: &ChatEngine,
    session_id: SessionId,
    nick: &str,
    msg: &IrcMessage,
) -> Vec<String> {
    let Some(channels_param) = msg.params.first() else {
        return vec![formatter::err_needmoreparams(nick, "PART")];
    };

    let reason = msg.params.get(1).cloned();
    let mut replies = Vec::new();

    for channel in channels_param.split(',') {
        let channel = channel.trim();
        if channel.is_empty() {
            continue;
        }

        if let Err(e) = engine.part_channel(session_id, channel, reason.clone()) {
            warn!(error = %e, %channel, "PART failed");
            replies.push(formatter::err_notonchannel(nick, channel));
        }
    }

    replies
}

fn handle_privmsg(
    engine: &ChatEngine,
    session_id: SessionId,
    nick: &str,
    msg: &IrcMessage,
) -> Vec<String> {
    if msg.params.len() < 2 {
        return vec![formatter::err_needmoreparams(nick, "PRIVMSG")];
    }

    let target = &msg.params[0];
    let content = &msg.params[1];

    if let Err(e) = engine.send_message(session_id, target, content) {
        warn!(error = %e, %target, "PRIVMSG failed");
        return vec![formatter::err_nosuchnick(nick, target)];
    }

    vec![]
}

fn handle_topic(
    engine: &ChatEngine,
    session_id: SessionId,
    nick: &str,
    msg: &IrcMessage,
) -> Vec<String> {
    let Some(channel) = msg.params.first() else {
        return vec![formatter::err_needmoreparams(nick, "TOPIC")];
    };

    if let Some(new_topic) = msg.params.get(1) {
        // Setting topic
        if let Err(e) = engine.set_topic(session_id, channel, new_topic.clone()) {
            warn!(error = %e, %channel, "TOPIC set failed");
            return vec![formatter::err_notonchannel(nick, channel)];
        }
        // The engine will broadcast TopicChange via events
        vec![]
    } else {
        // Querying topic — read from engine's channel state
        match engine.get_members(channel) {
            Ok(_) => {
                // Channel exists, get its topic by listing channels
                let channels = engine.list_channels();
                if let Some(ch) = channels.iter().find(|c| c.name == *channel) {
                    if ch.topic.is_empty() {
                        vec![formatter::rpl_notopic(nick, channel)]
                    } else {
                        vec![formatter::rpl_topic(nick, channel, &ch.topic)]
                    }
                } else {
                    vec![formatter::err_nosuchchannel(nick, channel)]
                }
            }
            Err(_) => vec![formatter::err_nosuchchannel(nick, channel)],
        }
    }
}

fn handle_names(engine: &ChatEngine, nick: &str, msg: &IrcMessage) -> Vec<String> {
    let Some(channel) = msg.params.first() else {
        return vec![formatter::err_needmoreparams(nick, "NAMES")];
    };

    match engine.get_members(channel) {
        Ok(member_infos) => {
            let nicks: Vec<String> = member_infos.iter().map(|m| m.nickname.clone()).collect();
            vec![
                formatter::rpl_namreply(nick, channel, &nicks),
                formatter::rpl_endofnames(nick, channel),
            ]
        }
        Err(_) => vec![formatter::rpl_endofnames(nick, channel)],
    }
}

fn handle_list(engine: &ChatEngine, nick: &str) -> Vec<String> {
    let channels = engine.list_channels();
    let mut replies = Vec::with_capacity(channels.len() + 1);

    for ch in &channels {
        replies.push(formatter::rpl_list(nick, &ch.name, ch.member_count, &ch.topic));
    }
    replies.push(formatter::rpl_listend(nick));

    replies
}

fn handle_who(engine: &ChatEngine, nick: &str, msg: &IrcMessage) -> Vec<String> {
    let Some(target) = msg.params.first() else {
        return vec![formatter::err_needmoreparams(nick, "WHO")];
    };

    let mut replies = Vec::new();

    if target.starts_with('#') {
        // WHO for a channel — list members
        if let Ok(members) = engine.get_members(target) {
            for member in &members {
                replies.push(format!(
                    ":{} {} {} {} {} {} {} {} H :0 {}",
                    formatter::server_name(),
                    super::numerics::RPL_WHOREPLY,
                    nick,
                    target,
                    member.nickname,
                    formatter::server_name(),
                    formatter::server_name(),
                    member.nickname,
                    member.nickname,
                ));
            }
        }
    }

    replies.push(format!(
        ":{} {} {} {} :End of /WHO list",
        formatter::server_name(),
        super::numerics::RPL_ENDOFWHO,
        nick,
        target,
    ));

    replies
}

fn handle_whois(engine: &ChatEngine, nick: &str, msg: &IrcMessage) -> Vec<String> {
    let Some(target) = msg.params.first() else {
        return vec![formatter::err_needmoreparams(nick, "WHOIS")];
    };

    if !engine.is_nick_available(target) {
        // User exists (nick is taken = user is online)
        vec![
            formatter::rpl_whoisuser(nick, target),
            formatter::rpl_whoisserver(nick, target),
            formatter::rpl_endofwhois(nick, target),
        ]
    } else {
        vec![formatter::err_nosuchnick(nick, target)]
    }
}

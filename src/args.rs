use clap::{App, Arg};
use serenity::model::prelude::*;

pub struct Args {
    pub token: Option<String>,
    pub guild_id: Option<GuildId>,
    pub channel_id: Option<ChannelId>,
}

pub fn get_args() -> Args {
    let matches = App::new("Terminal Image Viewer")
        .author("Noskcaj")
        .about("Shows images in your terminal")
        .arg(
            Arg::with_name("token")
                .long("token")
                .env("DISCORD_TOKEN")
                .help("Discord token to use, can also be set from DISCORD_TOKEN env var"),
        ).arg(
            Arg::with_name("guild_id")
                .long("guild_id")
                .takes_value(true)
                .help("ID of Guild to connect to"),
        ).arg(
            Arg::with_name("channel_id")
                .long("channel_id")
                .takes_value(true)
                .help("ID of channel to connect to"),
        ).get_matches();

    let token = matches.value_of("token").map(|tok| tok.to_owned());

    let guild_id = matches
        .value_of("guild_id")
        .and_then(|id| id.parse().ok())
        .map(GuildId);

    let channel_id = matches
        .value_of("channel_id")
        .and_then(|id| id.parse().ok())
        .map(ChannelId);

    Args {
        token,
        guild_id: guild_id,
        channel_id: channel_id,
    }
}

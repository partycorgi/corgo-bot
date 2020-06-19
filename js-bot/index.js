const Discord = require("discord.js");
const libhoney = require("libhoney");
const hny = new libhoney({
  writeKey: process.env.HONEYCOMB_TOKEN || "asf",
  dataset: "discord",
  serviceName: "corgo-discord-bot",
  transmission: "writer",
});

const CORGO_BOT_ID = "714618235458289804";
// Create an instance of a Discord client
const client = new Discord.Client();

/**
 * The ready event is vital, it means that only _after_ this will your bot start reacting to information
 * received from Discord
 */
client.on("ready", () => {
  console.log("corgo ready!");
});

// Create an event listener for messages
client.on("message", (message) => {
  let ev = hny.newEvent();
  ev.add({ eventType: "message", authorId: message.author.id });
  if (message.author.id === CORGO_BOT_ID) {
    ev.addField("isBot", true);
    ev.send();
    return;
  }

  if (message.content === "!avatar") {
    // If the message is "what is my avatar"
    // Send the user's avatar URL
    message.reply(message.author.displayAvatarURL());
    ev.addField("avatar", true);
  }
  ev.send();
});

hny.sendNow({ booting: true });
// Log our bot in using the token from https://discordapp.com/developers/applications/me
client.login(process.env.DISCORD_TOKEN);

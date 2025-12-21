# Musical Typing

This is a neat little project that aims to let users practice typing while listening to music. Currently there are only 2 main typing modes:

- type along to songs
- type the project source code

It also supports typing in other languages with dictionary files but currently only japanese is added.

Later on, I expect to add more modes to help users practice with other specific target text to either learn how to type, or we could source text from other open sources.

## Build Instructions

First install the repository either by downloading the zip or cloning it from the command line.

Next you could optionally connect to Spotify. This requires a Spotify developer account to get a clinet ID and client secret to put into the .env file. An example.env file exists to show how this should be done, and spotify has useful resources for creating an acount and getting these strings at the following links:

- [Spotify dashboard page](https://developer.spotify.com/dashboard)
- [Spotify developer guide](https://developer.spotify.com/documentation/web-api)


Because the project is made in [Rust](https://rust-lang.org/), a majority of the work for actually compiling the binary can be done by executing the following in the root folder of the project:

```
cargo run
```

## Tools used

This project is built in rust, using iced for windowing, reqwest and rspotify for API calls, as well as some other common rust dependencies for logging, serialization, and multithreading. It's loosely inspired by the website [keybr.com](keybr.com). I hope to add more progression features and feedback in the future to better align with that as a typing education tool. As mentioned above it uses the Spotify API for playback and searching of songs, and it also uses LRCLIB to fetch the lyrics you type along to.

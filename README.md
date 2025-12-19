# Musical Typing

This is a neat little project that aims to let users practice typing while listening to music. Currently there are only 2 main typing modes:
- type along to songs
- type the project source code

Later on I expect to add more modes to help users practice with other specific target text to either learn how to type, or we could source text from other open sources.

## Setting up the project 

Because the project is made in rust, a majority of the work can be done by installing the project into a local folder and then executing "cargo run"

Currently the app also depends on an integration with spotify so you will need to make a developer acount on spotify and create an app. then you can plug the client ID and client secret from that app into a file called ".env". (you could copy the example.env rename it to .env and replace the < angle brackets items > with your values)
Spotify has several guides on how to do this 
- [Spotify dashboard page](https://developer.spotify.com/dashboard)
- [Spotify developer guide](https://developer.spotify.com/documentation/web-api)

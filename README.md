![Alt text](./static/turnips_logo.svg)

Hi Turnips is an online exchange for Animal Crossing New Horizons turnips.

In the popular game, players can buy turnips on Sunday morning, and then sell them at Nook's cranny during the week - Tim & Tom prices oscillates during the weeks, making it difficult to earn a lot by selling them.

Hi Turnips wants to make things easier for the players: connecting to the online platform they can travel to other people's islands to sell turnips, or open the gates and let visitors come in for selling.

We're not online yet, but we'll publish the website soon. Stay tuned!

## Running the project

```
git clone https://github.com/danielabrozzoni/hiturnips.git
cd hiturnips
cargo run
```

The platform connects to a Redis database at `redis://127.0.0.1/`; this means that for making it work you'll need to start a [redis server](https://redis.io/download).
```
redis-server
```

Finally, visit `localhost:8000`

# raycasting-term
Raycasting in a terminal using rust

## Example
![Example image](https://user-images.githubusercontent.com/53996617/200564807-9fbe39f5-43cb-48b1-a121-ab4bf0b94c2e.png)

## Usage
### Singleplayer
```sh
cargo run
```

### Multiplayer
#### Server
```sh
cargo run --package=server PORT
```

#### Client
```sh
cargo run "SERVER_ADDRESS:SERVER_PORT"
```

## Client key bindings
| Key | Description |
| --- | --- |
| `Esc` | Exit the program |
| `W` | Move forward |
| `S` | Move backward |
| `A` | Strafe left |
| `D` | Strafe right |
| `Left` | Turn to the left |
| `Right` | Turn to the right |

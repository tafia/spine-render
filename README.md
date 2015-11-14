# spine-render

Rendering test for spine

## description

Renders spine files (.json skeleton, .atlas and .png for texture) using rust:
- [spine-rs](https://github.com/tafia/spine-rs/tree/skeleton) to parse and compute spine animations
- [glium](https://github.com/tomaka/glium) for opengl wrapper.

## usage

```
Spine renderer.

Usage:
  spine-render play [options] <json> <atlas> <png>
  spine-render list <json>
  spine-render (-h | --help)

Options:
  -h --help      Show this screen.
  --version      Show version.
  --fps <fps>    Frames per seconds [default: 60.0].
  --anim <anim>  Animation name [default: *].
  --skin <skin>  Skin name [default: default].
```

## license

MIT

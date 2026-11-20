## __Rust Adventure__

### __Name__: Chris Carson

### __Summary__:
Rust adventure (RAD) is a text-based adventure game in the style of MUDs, adventure, and Legend of the Red Dragon. The core game loop is to explore the dungeon, improve your character, and explore the dungeon some more. There is no deep storytelling, just a fun way to spend a little bit of time. RAD uses a more real time approach for attacks along with a skill system for traps and other events.

### __Installation and Running__:
It's rust, assuming you have everything installed:
`cargo run`

### __Approach__:
The original intent of RAD was to get some experience moving into Rust as a language and it kind of expanded from there. The initial idea I had was to work with a more MUD centric type of game, but I wanted something that would play faster and that I could just walk away from. That meant randomly generated content and a relatively short game loop. I picked a text UI framework for Rust, ratatui, to ease the process.
I designed the screen split and then did the UI layout. I moved from there to the "movement" handling between locations. Then worked on adding dynamic elements with data read from JSON files into hashmaps and make the database globally accessible. Currently the core game loop functions with an example combat. The game is maybe v0.5.

The current roadmap:
\
-Grant rewards for combat completion (decide on treasure packets, XP, etc)
-Add character improvement functions (new training, skills, spells, and abilities)
-Add alternate exploration events (traps and treasure rooms)
-Expand dungeon data to increase the depth of the dungeon (currently only 5 levels)

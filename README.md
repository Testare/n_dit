# `n_dit`

This is a project I've been working on in my freetime, primarily as a means of recreation and developing skills.

The minimum-viable output of this project is to recreate The Lego Spybotic's based flash game [The Nightfall Incident](https://brickipedia.fandom.com/wiki/The_Nightfall_Incident), but rather than using flash or even typical windowed interface, I want it to run in a terminal. Then, the goal is to use this point as a launching point for another game in the future, so some of the terminology of the project might not match exactly with nomenclature with the game (I.E. using "Curios" instead of "Programs" to describe game pieces)

Not long into the project I made the decision to swap from using pure rust code for the project to using [the Bevy game engine](https://www.github.com/bevyengine/bevy), primarily for the use of ECS. This provides many benefits such as performance gains, managed state to alleviate rust ownership pains, and the opportunity to develop libraries and contribute source code back to this open source project.

## Project Rules

* UI and game core code should be separate, opening the future of the project in case we want to have a GUI run on the project
* Code should be written under the assumption the game will eventually be multiplayer
* Some libraries should be written so that they can be split out into separate crates someday. Especially Charmi, which is our library for representing "Character Map Images" and associated processing, should be published to its own crate once it is mature enough. Either in the same crate or separate crates we should also include the code for using it in Bevy, especially if I figure out a good method of using GPUs and shaders to process them.
  

## Distant goals

Once the MVP has been met, the world opens up for further expansions:

* Creating a new game, using n_dit as a shared library for code.
* Multiplayer
* True CLI game edition: Playing the game through command-line commands in the terminal instead of an interactive TUI.
* GUI for the game
* Fan-expansion modules: New programs, maps, or even stories.

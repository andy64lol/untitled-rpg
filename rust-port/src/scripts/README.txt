UNTITLED RPG SCRIPT (.urpgs)

Script files live in src/scripts/ and use the .urpgs extension.
Comments start with # or //. Every statement ends with ;

COMMANDS (C-style calls, strings in quotes)
  dialogue("text");         Show dialogue and wait for ENTER/SPACE
  say("text");              Same as dialogue
  give("item_id", count);   Add items to the inventory
  take("item_id", count);   Remove items from the inventory
  gold(amount);             Add gold (negative removes it)
  spawn("enemy_id", c, r);  Spawn enemy (nearest spawnEnemy marker if no tile)
  movePlayer("right");      Hold a direction as synthetic WASD input (Rust replay)
  stopMove();               Release synthetic movement (Rust replay)
  face("up");               Turn without moving
  tick(30);                 Wait 30 frames (one action per frame)
  log();                    Log current player/enemy state
  log("text");              Log a custom message
  interact();               Perform the E interaction
  attack();                 Perform the Z attack
  flag("name");             Set a persistent flag
  unflag("name");           Clear a flag
  wait(seconds);            Pause the script
  run("scriptName");        Run another .urpgs script, then continue
  heal(amount);             Heal the player
  damage(amount);           Hurt the player
  setHealth(amount);        Set player health
  setGold(amount);          Set gold
  end();                    Stop the script

PLAYER API IN CONDITIONS
  player.health             Current HP        (also player.hp)
  player.maxHealth          Max HP            (also player.maxhp)
  player.dead               True when dead
  player.moving             True while moving
  player.facing             "up"/"down"/"left"/"right"
  player.gold               Current gold
  player.item <id>          Item count
  player.map                Bare map name
  Operators: < > <= >= == !=   e.g. if (player.health < 50) { ... }

LOGIC (C-style braces)
  if (item <id> [count]) { ... } else { ... }
  if (gold <amount>) { ... }
  if (flag <name>) { ... }
  if (map map1) { ... }     Bare map name, no .tmx
  else if (...) works too.

EXAMPLE
  dialogue("Welcome!");
  give("apple", 3);
  if (flag met_wizard) {
    dialogue("You already met me.");
  } else {
    dialogue("Nice to meet you.");
  }

AUTO-START
  init.urpgs runs automatically whenever a new game is started.

TRIGGERS
  Place a map object named "script:<name>" to run scripts/<name>.urpgs
  when the player faces and interacts with it (E).
  Run one at startup with:  --script <name>

PLAYER-MOVE REPLAY (RUST ONLY)
  --player-moves <name> loads scripts/<name>.urpgs and starts the game immediately.
  The extension is optional, so both "moves" and "moves.urpgs" work.
  The script's movePlayer()/tick()/stopMove() calls drive the player like WASD.
  The default init script is skipped unless --script is also passed.
  See moves.urpgs for an example. Enemy spawns are logged with source and tile.
  Triggers arm on the marker and fire once the player is enemy_spawn_distance_tiles
  (default 2.0) tiles away. Returning to the marker keeps it armed.
  New enemies spawn on the spawnEnemy point and start moving immediately.
  Enemies and the player are solid to each other; neither can walk onto the
  other's tile, and enemies attack from an adjacent tile.
  damage_speed is tenths of a second between hits, so higher values attack slower.
  Enemy damage/defense/hp come from enemies.json ("damage" is attack). Your
  equipped attack minus enemy defense reduces enemy HP; hits show a red number
  and flash the enemy red, and 0 HP starts the death animation.
  Damage logs show source, enemy spawn and current positions, raw/dealt damage,
  HP, interval since the previous damage event, and the next cooldown.
  Spawn markers are numbered numerically (spawnEnemy2 before spawnEnemy10).

Item ids are from items.json. Enemy ids are from enemy.json ("id").

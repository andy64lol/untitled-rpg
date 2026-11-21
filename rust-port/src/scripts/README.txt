UNTITLED RPG SCRIPT (.urpgs)

Script files live in src/scripts/ and use the .urpgs extension.
Comments start with # or //. Every statement ends with ;

COMMANDS (C-style calls, strings in quotes)
  dialogue("text");         Show dialogue and wait for ENTER/SPACE
  say("text");              Same as dialogue
  give("item_id", count);   Add items to the inventory
  take("item_id", count);   Remove items from the inventory
  gold(amount);             Add gold (negative removes it)
  spawn("enemy_id", c, r);  Spawn enemy (next to player if no tile)
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

Item ids are from items.json. Enemy ids are from enemy.json ("id").

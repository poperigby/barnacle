# Objects
object_game = { $count ->
    [1] Game
   *[other] Games
}
object_profile = { $count ->
    [1] Profile
   *[other] Profiles
}
object_mod = { $count ->
    [1] Mod
   *[other] Mods
}
object_tool = { $count ->
    [one] Tool
   *[other] Tools
}

# Fields
name = Name
path = Path

# Common actions
add = Add
confirm = Confirm
create = Create
cancel = Cancel

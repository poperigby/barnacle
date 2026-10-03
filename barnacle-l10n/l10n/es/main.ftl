# Objects
object_game = { $count ->
    [1] Juego
   *[other] Juegos
}
object_profile = { $count ->
    [1] Perfil
   *[other] Perfiles
}
object_mod = { $count ->
    [1] Mod
   *[other] Mods
}
object_tool = { $count ->
    [one] Herramienta
   *[other] Herramientas
}

# Fields
name = Nombre
path = Ruta

# Chasedow : tâches et contexte

Ce document est autosuffisant : chaque tâche contient ce qu'il faut pour la coder sans relire tout le code. Numéros de ligne relatifs à `src/main.rs` au commit `77b5d84` ; ils bougent, cherche plutôt le nom de la fonction.

Principes : rester simple, ne pas casser l'existant, ne pas changer l'esprit du jeu (course sans fin contre son ombre). Un commit par tâche. Vérifier avec `cargo build` puis `cargo run --release` (le CI lance `cargo build` et `cargo test`).

---

## 0. Contexte commun (à lire une fois)

### Architecture
Tout est dans `src/main.rs` (~906 lignes), assets dans `assets/` : `player.png` (spritesheet unique, cases de 12×12 px), `background.ogg`, `screen.png` (capture du README). Il n'y a pas d'autre son.

Dépendances : `macroquad` 0.4.13, `macroquad-platformer` (physique : `World`, `Actor` pour le joueur, `Solid` pour les plateformes), `rand`, `futures`, `getrandom` (feature `js`, donc cible WASM envisagée).

### Boucle principale
`main()` : `game.update().await` puis `game.draw()` puis `next_frame().await`.

`GameState` (ligne ~105) est le seul état global. Champs : `world`, `player`, `shadow`, `platforms`, `score: f32` (secondes survécues), `screen: GameScreen`, `high_score: f32`, `lives: i32`, `invulnerable_timer: f32`, `is_invulnerable: bool`, `audio: GameAudio`, `coins: Vec<Coin>`, `coin_spawn_timer: f32`, `coin_points: i32`.

`GameScreen` : `MainMenu`, `Playing`, `Paused`, `GameOver`. `update()` et `draw()` font un `match` vers `update_*` / `draw_*`.

Fonctions clés de `GameState` :
- `new()`, `reset_game()` : (ré)initialisation.
- `update_playing()` : logique d'une frame, voir ci-dessous.
- `handle_shadow_collision()` : retire une vie, passe en `GameOver` si `lives <= 0`.
- `spawn_coin()` : position aléatoire, utilise `block_on` pour créer la pièce.
- `draw_playing()`, `draw_ui()`, `draw_lives()`, `draw_main_menu()`, `draw_game_over()`, `draw_paused()`.
- `should_draw_player()` : clignotement pendant l'invulnérabilité.

Ordre dans `update_playing()` : timer d'apparition des pièces, mise à jour et ramassage des pièces, timer d'invulnérabilité, pause (Échap), `platform.update`, `player.update`, bornes de la fenêtre, `shadow.update(player_pos)`, collision ombre/joueur, `score += get_frame_time()`.

Ordre dans `draw_playing()` : pièces, plateformes, ombre, joueur (si `should_draw_player`), puis `draw_ui()`.

### Mécanique de l'ombre (centrale)
`Shadow` (ligne ~639) garde `positions: Vec<Vec2>` de taille fixe `delay_frames`. Chaque frame, `update(player_pos)` fait `remove(0)` puis `push(player_pos)`. L'ombre est dessinée et testée en collision à `positions.first()`, donc elle rejoue la position du joueur d'il y a `delay_frames` frames.
- Le délai est en frames, pas en secondes : il dépend du framerate (à 60 FPS, 75 frames = 1,25 s).
- `collides_with_player` utilise une hitbox réduite (`PLAYER_SIZE - 16` px) pour compenser les marges du sprite.
- `Shadow::draw` choisit marche ou saut en comparant `last_removed_position.y` et `pos.y`.

### Physique et entrées
- Joueur : `Actor` de 48×48 (`PLAYER_SIZE`), départ en (250, 500). `Player::update` calcule `on_ground` via `world.collide_check(collider, pos + (0,1))`, puis `handle_movement(on_ground)` (gravité, flèches gauche/droite, saut sur `Space` si `on_ground`), puis `apply_movement` (`move_h` / `move_v`).
- Constantes : `GRAVITY 500`, `PLAYER_SPEED 150`, `JUMP_FORCE -350`, `PLATFORM_SPEED 50`.
- Plateformes : `Solid` de 200×12. `create_platforms()` en définit 8 avec positions en dur : 2 mobiles (`is_moving = true`, va-et-vient en x entre 150 et 500), 5 statiques, 1 sol (800×12 à y = 585). Les cactus ne sont dessinés que sur les statiques.
- Fenêtre 800×600 (`WINDOW_WIDTH` / `WINDOW_HEIGHT`).

### Pièces
`Coin` : `position`, `lifetime` (5 s), texture. `update()` renvoie `false` quand elle expire. Elle clignote sous 1 s. Apparition toutes les `COIN_SPAWN_INTERVAL` = 3 s, en (x aléatoire, y entre 100 et 535). Ramassage : `coin_points += COIN_POINTS` (10), affiché en « Coins: N ».

### Style
Constantes de couleur en haut du fichier (`TEXT_PRIMARY`, `TEXT_ACCENT`, `TEXT_GOLD`, etc.), palette désert/sépia. Le texte se dessine avec `draw_text` et se centre via `measure_text`. Réutiliser ces constantes plutôt qu'inventer des couleurs.

### Audio
`GameAudio` ne charge que `background.ogg`. `play_background()` existe mais son appel est commenté dans `update_playing` (« fixme macroquad issue 440 »), donc la musique ne se joue pas aujourd'hui. Les sons de saut et de game over sont commentés dans la struct.

### Faits à connaître avant de coder
- **La difficulté est écrasée au redémarrage** : `reset_game()` fait `Shadow::new(25)` alors que le départ utilise `SHADOW_FRAMES_DELAY` (75). Après le premier restart, l'ombre est donc beaucoup plus proche (voir B1).
- Le high score n'est mis à jour que dans `reset_game()`. En quittant le game over avec Échap vers le menu, il n'est pas mis à jour (voir tâche 1).
- `coin_points` est séparé de `score` : les pièces ne comptent pas dans le score affiché au game over.
- `reset_game()` ne remet pas `is_invulnerable` à `false`.
- `Coin::new`, `Shadow::new`, `Platform::new` rechargent chacun `player.png` (une texture par instance, et par pièce spawnée). Fonctionne, mais gaspille ; ne pas « corriger » en passant, hors périmètre.
- Il n'existe aucun test. Le CI lance `cargo test` : il passe sans rien.

---

## Bugs existants (à traiter en premier, petits et sûrs)

### B1. L'ombre démarre trop proche après un restart (FAIT, 06f7d27)
- **Cause** : `reset_game()` appelle `Shadow::new(25)` au lieu de `Shadow::new(SHADOW_FRAMES_DELAY)`.
- **À faire** : remplacer `25` par `SHADOW_FRAMES_DELAY`. Si la tâche 2 (difficulté progressive) est faite, utiliser la valeur initiale.
- **Attention** : c'est un changement de difficulté perceptible, le dire dans le message de commit. Le jeu devient plus facile après un restart, comme au premier lancement.
- **Fait quand** : le délai est identique au premier lancement et après un restart.

### B2. `is_invulnerable` non remis à zéro au restart (FAIT, e00d92d)
- **À faire** : ajouter `self.is_invulnerable = false;` dans `reset_game()`.
- **Fait quand** : mourir pendant l'invulnérabilité puis rejouer ne fait pas clignoter le joueur.

---

## Petits gains

### 1. High score persistant (FAIT)
- **But** : conserver le meilleur score entre les sessions et le mettre à jour correctement.
- **Contexte** : `high_score: f32` vit en mémoire, initialisé à `0.0` dans `GameState::new()`. Il est mis à jour dans `reset_game()` seulement. Il est affiché dans `draw_main_menu` (si > 0), `draw_ui` et `draw_game_over` (« New High Score! » si `score > high_score`).
- **À faire** :
  1. Déplacer la mise à jour du high score au moment où l'écran passe à `GameOver` (dans `handle_shadow_collision`, branche `lives <= 0`). Attention : `draw_game_over` teste `score > high_score` pour afficher « New High Score! ». Mettre à jour `high_score` avant fait disparaître ce message. Solution simple : ajouter un champ `new_record: bool` posé au game over, lu par `draw_game_over`, remis à `false` dans `reset_game()`.
  2. Charger au démarrage (`GameState::new`) et sauver à chaque nouveau record.
  3. Natif : `std::fs::read_to_string` / `std::fs::write` sur `highscore.txt`, avec parsing tolérant (`.parse::<f32>().unwrap_or(0.0)`, aucune panique si le fichier manque). Ajouter `highscore.txt` au `.gitignore`.
  4. WASM : `std::fs` ne marche pas. Soit ignorer le WASM dans cette tâche (`#[cfg(not(target_arch = "wasm32"))]`), soit ajouter la crate `quad-storage` plus tard (voir tâche 19).
- **Fait quand** : tuer le jeu et le relancer affiche l'ancien record sur le menu ; un record battu puis Échap vers le menu met bien à jour « High Score ».

### 2. Difficulté progressive
- **But** : l'ombre se rapproche avec le temps.
- **Contexte** : voir « Mécanique de l'ombre ». `positions` a une taille fixe égale à `delay_frames`. Pour réduire le délai de `n`, il faut retirer `n` éléments du début de `positions` et décrémenter `delay_frames` (sinon la taille et le délai divergent).
- **À faire** :
  1. Ajouter `Shadow::reduce_delay(&mut self, n: usize, min: usize)` : si `delay_frames - n >= min`, faire `self.positions.drain(0..n)` et `self.delay_frames -= n`.
  2. Constantes : `SHADOW_DELAY_STEP = 5`, `SHADOW_DELAY_MIN = 30`, `SHADOW_DELAY_INTERVAL = 30.0` (secondes).
  3. Dans `GameState`, un compteur `difficulty_timer: f32` décrémenté dans `update_playing`, remis à `SHADOW_DELAY_INTERVAL` après chaque palier, et dans `reset_game()`.
- **Pièges** : ne jamais descendre sous `SHADOW_DELAY_MIN` (sinon collision permanente avec le joueur). Le `drain` fait sauter l'ombre vers l'avant de `n` frames : c'est voulu, mais tester que ça ne tombe pas sur le joueur à l'instant du palier (l'invulnérabilité de 3 s couvre le cas, mais à vérifier).
- **Dépend de** : B1 (la valeur de départ du délai).
- **Fait quand** : à 30 s, 60 s, etc., l'ombre est visiblement plus proche ; elle plafonne ; un restart repart à 75.

### 3. Feedback à la prise d'une pièce
- **But** : petit texte « +10 » qui monte et s'efface, et un son si possible.
- **Contexte** : le ramassage est dans la boucle `while i < self.coins.len()` de `update_playing` (`coin_points += COIN_POINTS`). Il n'y a pas de son de pièce (voir « Audio »).
- **À faire** :
  1. Struct `FloatingText { pos: Vec2, life: f32, text: String }` ; `Vec<FloatingText>` dans `GameState`, vidé dans `reset_game()`.
  2. Au ramassage : push avec la position de la pièce.
  3. Dans `update_playing` : `pos.y -= 30.0 * get_frame_time()`, `life -= get_frame_time()`, puis `retain(|t| t.life > 0.0)`.
  4. Dans `draw_playing` (avant `draw_ui`) : `draw_text` en `TEXT_GOLD`, alpha = `life / durée`.
  5. Son : si on ajoute `assets/coin.ogg`, le charger dans `GameAudio::new()` et ajouter `play_coin()` sur le modèle de `play_background` (`looped: false`). Si on n'a pas de fichier, sauter cette partie ; ne pas inventer un asset.
- **Fait quand** : chaque pièce ramassée fait apparaître un « +10 » qui monte et disparaît en environ 0,7 s.

### 4. Corriger l'audio (musique inactive)
- **But** : que la musique se joue vraiment.
- **Contexte** : l'appel `self.audio.play_background()` est commenté dans `update_playing`, avec un fixme vers l'issue macroquad 440. Appelé à chaque frame il relancerait le son en boucle, d'où la mise en commentaire. `play_background` a aussi son `is_sound_playing` commenté.
- **À faire** : jouer la musique une seule fois, à la transition vers `Playing` (dans `update_main_menu` et `update_game_over`, après `reset_game`), et `stop_all()` au game over. Ne pas appeler dans la boucle de frame. Gérer la pause : soit laisser tourner, soit `stop_all()` puis relance (choisir le plus simple, laisser tourner).
- **Attention** : sur navigateur, l'audio exige une interaction utilisateur préalable ; le déclencher sur l'appui de Espace du menu règle ça.
- **Fait quand** : la musique démarre à l'appui de Espace, ne se superpose pas au restart, et s'arrête au game over. Si ça ne marche pas sous Linux, noter le comportement observé au lieu de forcer.

---

## Effets sur l'ombre

### 5. Traînée de l'ombre
- **But** : donner de la substance à l'ombre (roadmap du README).
- **Contexte** : `Shadow::draw` dessine un seul sprite à `positions.first()`. Le vecteur `positions` contient tout l'historique entre l'ombre et le joueur, donc les frames « entre » sont déjà là.
- **À faire** : avant le sprite principal, dessiner 2 ou 3 copies plus transparentes aux indices `positions[k * 4]` (k = 1..3), avec `Color::new(1.0, 1.0, 1.0, 0.25 / k as f32)` et la même `source` / `flip_x`. Garder `dest_size` à `PLAYER_SIZE`. Ne pas toucher à `collides_with_player` (la hitbox reste à `first()`).
- **Piège** : vérifier `positions.len() > k * 4` pour éviter une panique si le délai est réduit (tâche 2).
- **Fait quand** : on voit une queue fantomatique derrière l'ombre, sans changement de difficulté.

### 6. Ombre qui réagit à la proximité
- **But** : avertir le joueur quand l'ombre est proche.
- **Contexte** : `shadow.positions.first()` et `player_pos` sont tous deux disponibles dans `update_playing`.
- **À faire** : calculer `dist = player_pos.distance(shadow_pos)`. En dessous d'un seuil (≈ 150 px), dessiner un cercle d'alerte discret ou teinter l'ombre (`Color` du `draw_texture_ex` en rouge-brun). Pas de nouvelle mécanique, uniquement visuel.
- **Fait quand** : l'ombre change visiblement d'aspect quand elle approche, et redevient normale sinon.

### 7. Flash ou secousse à la touche
- **But** : retour visuel quand on perd une vie.
- **Contexte** : `handle_shadow_collision` retire la vie et lance `invulnerable_timer`. Le clignotement existe déjà (`should_draw_player`), mais pas d'effet global.
- **À faire** : champ `hit_flash: f32` posé à 0.2 s dans `handle_shadow_collision` (branche où le joueur survit) et décrémenté dans `update_playing`. Dans `draw_playing`, avant `draw_ui`, dessiner un `draw_rectangle` plein écran rouge avec alpha = `hit_flash * 2.0`. Option screen shake : `Camera2D` décalée aléatoirement, puis `set_default_camera()` avant `draw_ui` pour que l'UI ne tremble pas.
- **Piège** : remettre `hit_flash` à 0 dans `reset_game()`.
- **Fait quand** : un flash rouge bref apparaît à chaque vie perdue.

---

## Gameplay simple

### 8. Pièce spéciale
- **But** : un peu de risque et de récompense.
- **Contexte** : `Coin` n'a ni type ni valeur propre ; la valeur est la constante `COIN_POINTS`. Le sprite normal est la case (4, 2) de la spritesheet, de 12×12.
- **À faire** : ajouter `value: i32` à `Coin` (pour `COIN_POINTS` ou 50) et une durée de vie plus courte pour la spéciale (2,5 s). Dans `spawn_coin`, tirer la spéciale avec une probabilité de 10 % (`gen_range(0, 10) == 0`). Si on n'a pas de sprite dédié, teinter avec `TEXT_GOLD` plutôt que d'ajouter un asset. Dans la boucle de ramassage, utiliser `coins[i].value`.
- **Fait quand** : on voit parfois une pièce dorée qui disparaît vite et vaut plus.

### 9. Power-up : ombre ralentie
- **But** : premier power-up, dans le thème de l'ombre.
- **Contexte** : à voir avec la tâche 2 (`delay_frames`). Il n'y a pas de système de power-up : il faut un type d'objet ramassable.
- **À faire** : réutiliser le mécanisme des `Coin` avec un champ `kind` (enum `Normal`, `Special`, `SlowShadow`). À la collecte de `SlowShadow` : allonger le délai de l'ombre de +30 frames pendant 5 s, via un `Vec<Vec2>` agrandi en dupliquant la dernière position (ou, plus simple, geler l'ombre : ne pas appeler `shadow.update` pendant 2 s). **Recommandé : le gel**, il ne touche pas à la structure de `positions`. Timer `shadow_freeze: f32` dans `GameState`, remis à 0 dans `reset_game()`.
- **Piège** : si l'ombre est gelée, elle reste à `positions.first()`, donc elle peut toucher le joueur qui revient dessus. Pendant le gel, désactiver aussi la collision ombre/joueur, sinon c'est injuste.
- **Fait quand** : ramasser l'objet fige l'ombre pendant la durée, avec un indicateur à l'écran.

### 10. Power-up : vie bonus
- **But** : rattrapage rare.
- **Contexte** : `lives` est un `i32` avec `INITIAL_LIVES = 3` ; `draw_lives` dessine `INITIAL_LIVES` cœurs (rouges si `i < lives`, gris sinon), donc ne gère pas plus de 3.
- **À faire** : `kind = ExtraLife`, rare (≈ 3 % des spawns). À la collecte : `self.lives = (self.lives + 1).min(INITIAL_LIVES)`. Ne pas dépasser 3, sinon `draw_lives` ne suit pas.
- **Fait quand** : on ne peut jamais avoir plus de 3 vies, et ramasser l'objet à 3 vies ne casse rien.

### 11. Combo de pièces
- **But** : récompenser l'enchaînement.
- **Contexte** : `coin_points` s'incrémente de `COIN_POINTS` ; le score (`score`) ne compte pas les pièces. Choisir d'abord si les pièces doivent compter dans le score final : c'est une décision de design. Par défaut, **ne rien changer au score** et ne travailler que sur `coin_points`.
- **À faire** : `combo: i32` et `combo_timer: f32` (2 s) dans `GameState`. À chaque pièce : si `combo_timer > 0`, `combo += 1`, sinon `combo = 1` ; puis `combo_timer = 2.0` ; points = `COIN_POINTS * combo.min(5)`. Décrémenter `combo_timer` dans `update_playing`. Afficher « xN » dans `draw_ui`. Remettre à zéro dans `reset_game()`.
- **Fait quand** : trois pièces en moins de 2 s chacune donnent 10, 20, 30.

### 12. Plateformes qui disparaissent
- **But** : nouveau type de plateforme (roadmap).
- **Contexte** : `Platform` a `collider: Solid`, `speed`, `size`, `cacti`. `is_moving` est un booléen passé à `Platform::new`. Pour qu'une plateforme disparaisse, il faut la retirer de la physique : `Solid` n'a pas de flag « inactif », mais on peut le déplacer hors écran avec `world.set_solid_position` (et le remettre à sa position d'origine ensuite).
- **À faire** : champ `kind: PlatformKind` (`Static`, `Moving`, `Fading`) en remplaçant le booléen, ou ajouter un second booléen `is_fading` pour minimiser les changements. Pour `Fading` : état (`Solid`, `Warning(timer)`, `Gone(timer)`). Le joueur doit toucher la plateforme pour lancer le compte : détecter avec `world.collide_check(player.collider, player_pos + (0,1))` sur ce solide, c'est plus simple de tester `player_rect` juste au-dessus du solide. Disparition 1 s après contact, retour 3 s plus tard. Rendu : alpha réduit pendant `Warning`, rien pendant `Gone`.
- **Pièges** : le joueur posé dessus au moment de la disparition tombe : c'est voulu. Ne pas mettre de plateforme fragile en sol (`y = 585`) : le joueur ne pourrait plus se rattraper. Les cactus ne se dessinent que si `speed == 0.0` : à adapter pour la nouvelle plateforme.
- **Fait quand** : au moins une des plateformes de `create_platforms` est fragile et le jeu reste jouable.

---

## Qualité de vie

### 13. Contrôles alternatifs
- **But** : jouer à A/D ou Q/D en plus des flèches.
- **Contexte** : `Player::handle_movement` lit `is_key_down(KeyCode::Right)` / `Left`, et `is_key_pressed(KeyCode::Space)` pour le saut. Le menu (`draw_main_menu`) affiche « LEFT/RIGHT - Move ».
- **À faire** : `let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);` et `let left = ... || is_key_down(KeyCode::A) || is_key_down(KeyCode::Q);`. Sur AZERTY, la touche « Q » physique est mappée selon la disposition : tester les deux. Mettre à jour le texte des contrôles du menu et le README.
- **Fait quand** : les flèches, A/D et Q/D déplacent le joueur ; le menu l'indique.

### 14. Couper la musique (touche M)
- **But** : contrôle du volume minimal.
- **Contexte** : `GameAudio` ne gère que `background_music`, avec un volume de 0,5 en dur. Dépend de la tâche 4 (la musique doit jouer pour être coupée).
- **À faire** : champ `muted: bool` dans `GameAudio`, touche `KeyCode::M` testée dans `GameState::update` (quel que soit l'écran). Basculer avec `set_sound_volume(&sound, 0.0 / 0.5)`. Afficher « M: sound on/off » dans le menu.
- **Fait quand** : M coupe et rétablit la musique, y compris en pause et au menu.

### 15. Saut plus agréable (coyote time et saut tampon)
- **But** : meilleur ressenti, sans changer la physique.
- **Contexte** : `Player::handle_movement(on_ground)` accepte le saut seulement si `is_key_pressed(Space) && on_ground`. `Player` a `speed`, `size`, `texture`, `sprite`. `get_frame_time()` donne le dt.
- **À faire** :
  1. Champs `coyote_timer: f32` et `jump_buffer: f32` dans `Player` (init à 0.0 dans `Player::new`).
  2. Dans `handle_movement` : si `on_ground`, `coyote_timer = 0.1`, sinon `coyote_timer -= dt`. Si `is_key_pressed(Space)`, `jump_buffer = 0.1`, sinon `jump_buffer -= dt`.
  3. Si `jump_buffer > 0.0 && coyote_timer > 0.0` : `speed.y = JUMP_FORCE`, puis `jump_buffer = 0.0` et `coyote_timer = 0.0` (empêche le double saut).
- **Pièges** : l'animation de saut se règle sur `on_ground` (`set_animation(1)` quand `!on_ground`) : ne pas la changer, sinon le sprite ment. Les valeurs doivent rester petites (≤ 0,12 s) pour ne pas dénaturer le jeu.
- **Fait quand** : on peut sauter un instant après avoir quitté une plateforme, un appui juste avant l'atterrissage déclenche le saut, et on ne peut pas sauter en l'air.

---

## Technique

### 16. Découper `main.rs` en modules
- **But** : lisibilité, sans changer le comportement.
- **Contexte** : 906 lignes. Les constantes sont en tête (lignes 8 à 46). `Player`, `Shadow`, `Platform`, `Coin`, `GameAudio` sont des structs indépendantes ; seule `GameState` les orchestre.
- **À faire** : créer `src/constants.rs`, `audio.rs`, `player.rs`, `shadow.rs`, `platform.rs`, `coin.rs`, `game.rs` (`GameState` et `GameScreen`). Ne laisser que `main()` dans `main.rs`. Marquer `pub` ce qui est utilisé entre modules ; `use crate::constants::*;` partout.
- **Règle** : un module par commit, `cargo build` entre chaque, aucun changement de logique. Faire cette tâche **avant** les gros ajouts de fonctionnalités, ou à la toute fin, mais pas au milieu (conflits).
- **Fait quand** : le jeu se comporte à l'identique et `main.rs` fait moins de 20 lignes.

### 17. Tests unitaires
- **But** : garantir la logique pure et donner un sens à `cargo test` du CI.
- **Contexte** : la plupart du code dépend de macroquad (fenêtre, textures), donc non testable tel quel. Seule la logique de `Shadow` est pure en pratique : `Shadow::new` charge une texture et ne peut donc pas s'appeler en test.
- **À faire** : extraire la logique de délai dans une petite fonction ou struct sans dépendance graphique (ex. `ShadowTrail { positions, delay_frames }` avec `push(pos) -> Vec2`, `reduce_delay`), puis la tester : taille constante après `push`, `reduce_delay` respecte le minimum, la valeur retournée est bien celle d'il y a `delay` frames. Ajouter aussi un test de la formule de combo (tâche 11) si elle est extraite.
- **Dépend de** : tâche 2 pour `reduce_delay`.
- **Fait quand** : `cargo test` exécute au moins 3 tests utiles et passe.

### 18. Publication WASM (GitHub Pages)
- **But** : jeu jouable dans le navigateur.
- **Contexte** : `getrandom` est déjà configuré avec la feature `js`. Le CI actuel (`.github/workflows/rust.yml`) fait seulement `cargo build` et `cargo test` sur push/PR vers `master`. Les assets se chargent par `set_pc_assets_folder("assets")` : sur le web, macroquad les cherche par URL, donc `assets/` doit être servi à côté du `.wasm`.
- **À faire** : `rustup target add wasm32-unknown-unknown`, `cargo build --release --target wasm32-unknown-unknown`, copier le `.wasm`, un `index.html` minimal (chargeur `mq_js_bundle.js` de macroquad) et `assets/` dans un dossier `dist/`, puis un second workflow qui publie `dist/` sur GitHub Pages.
- **Pièges** : `block_on` dans `spawn_coin` (`futures::executor`) bloque le thread et peut poser problème sur WASM : à vérifier, c'est un risque réel. La sauvegarde du high score (tâche 1) ne marche pas sur WASM sans `quad-storage`. L'audio exige une interaction préalable (voir tâche 4).
- **Fait quand** : la page publiée démarre le jeu et le joue sans erreur console.

---

## Ordre recommandé

1. B1, B2 (bugs).
2. 1 (high score), 2 (difficulté), 3 (feedback pièces), 15 (saut).
3. 5 (traînée), puis 9 (premier power-up).
4. 4 et 14 (audio), 13 (contrôles).
5. 16 (modules) et 17 (tests) quand le code est stabilisé.
6. 18 (WASM) en dernier.

À éviter pour rester dans l'esprit : niveaux multiples, décors changeants, armes. Un seul décor en boucle colle mieux à une course sans fin.

## Hors périmètre (ne pas faire en passant)
- Refactorer le chargement de texture dupliqué (`player.png` chargé par instance).
- Corriger les marges du sprite (`fixme` dans `Shadow::collides_with_player`) : toucherait à la difficulté.
- Changer le thème ou la palette de couleurs.

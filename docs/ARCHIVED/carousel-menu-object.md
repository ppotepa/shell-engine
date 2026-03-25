# Centered Menu Composition

Ten dokument opisuje aktualny wzorzec budowania menu karuzelowego.

## 1) Granice odpowiedzialności

Scena:

- `menu-options` (routing po klawiszach),
- lifecycle (`stages`/`stages-ref`, `next`).
- dla interaktywnego menu: `on_idle.trigger: any-key`.

Layout menu:

- sprite tree (`grid`/`flex`/`text`) albo reusable object.

Runtime behavior:

- pozycjonowanie i widoczność elementów względem `menu.selected_index`.

## 2) Dane wejściowe menu

Wszystkie przejścia są deklarowane w `menu-options`:

```yaml
menu-options:
  - { key: "1", label: "START", to: scene-start }
  - { key: "2", label: "OPTIONS", to: scene-options }
  - { key: "3", label: "EXIT", to: scene-exit }
```

`to` jest canonical aliasem routingu.

Obsługiwane wejście runtime:

- `Up`/`Down` (zmiana aktywnej pozycji),
- `Enter` (aktywacja pozycji aktywnej),
- klawisze przypisane w `menu-options[].key`.

## 3) Wariant behavior wbudowany

Behaviory:

- `menu-carousel-object` (pozycjonuje listę),
- `selected-arrows` (wskaźnik `>`/`<` dla aktywnej pozycji).

Kluczowe parametry:

- `target`,
- `item_prefix`,
- `count`,
- `window`,
- `step_y`,
- `endless`.

## 4) Wariant behavior skryptowy Rhai

Skrypt Rhai emituje komendy runtime:

- `visibility`,
- `offset`.

Wspierane są też mutacje path-based:

- `set` (`target`, `path`, `value`),
- API obiektowe `scene.get(...).set(...)` oraz `scene.set(...)`.
- Dla tekstu: `text.content`, `text.font`, `style.fg`, `style.bg`.
- Dla obiektów 3D: `obj.scale`, `obj.yaw`, `obj.pitch`, `obj.roll`, `obj.orbit_speed`, `obj.surface_mode`, `obj.clip_y_min`, `obj.clip_y_max`.

Dostępne wartości w scope:

- `menu.selected_index`,
- `menu.count`,
- `time.scene_elapsed_ms`,
- `time.stage_elapsed_ms`,
- `params`,
- `regions`.
- `objects`,
- `state`.

Ten wariant jest używany m.in. przez:

- `mods/playground/scenes/menu/menu.rhai`,
- `mods/test-scenes/scenes/menu/menu.rhai`.
- `mods/shell-quest/behaviors/portrait-materialize.yml` — scanline materialize.

### portrait-materialize

Behavior `portrait-materialize` kontroluje efekt przejścia wireframe→solid
na portretach menu difficulty. Parametry: `index` (pozycja menu), `dur` (czas
animacji w ms, domyślnie 250).

Pipeline animacji:

1. **Glitch phase** (0–90ms): oba sprite'y (wire + solid) migają między
   wireframe a material z 25ms interwałem.
2. **Scanline phase** (90ms–dur): scanline zjeżdża w dół — wire `clip_y_min`
   rośnie, solid `clip_y_max` rośnie. Nad linią = solid, pod = wireframe.
3. **Done** (elapsed ≥ dur): wire ukryty (`clip_y_max=0`), solid widoczny
   (`clip_y_max=1`), oba na `yaw=0`.

Oba obroty (wire + solid) są synchronizowane: `yaw = 180 - (180 * progress)`.

Referencja: `mods/shell-quest/objects/difficulty-menu.yml`.

Uwagi praktyczne:

- skrypt dostaje regiony tylko dla istniejących aliasów (`target`, `item_prefix + index`),
- przy własnym pozycjonowaniu pionowym trzeba zapewnić krok bez kolizji między itemami (np. `step_y` >= wysokość itemu + 1).

## 5) Wzorzec reusable object

Reusable object może zawierać:

- kontener `grid`,
- itemy `menu-item-*`,
- sprite wskaźników.

Przykład referencyjny:

- `mods/playground/objects/centered-endless-menu.yml`,
- `mods/test-scenes/objects/centered-endless-menu-3.yml`.

## 6) Własności, które wzorzec gwarantuje

- wybrana pozycja jest utrzymywana w centrum okna menu,
- `window` ogranicza widoczne elementy,
- `endless: true` daje wrap-around jak bęben,
- minimalny krok pionowy respektuje wysokość renderowanego elementu.

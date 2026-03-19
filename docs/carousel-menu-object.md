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

- pozycjonowanie i widoczność elementów względem `selected_index`.

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

Dostępne wartości w scope:

- `selected_index`,
- `menu_count`,
- `scene_elapsed_ms`,
- `stage_elapsed_ms`,
- `params`,
- `regions`.

Ten wariant jest używany m.in. przez:

- `mods/playground/scenes/menu/menu.rhai`,
- `mods/test-scenes/scenes/menu/menu.rhai`.

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

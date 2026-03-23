Cutout

`cutout` to wbudowany efekt kolorystyczny działający po rasteryzacji bufora komórek terminalowych. W aktualnym runtime:

- przetwarza gotowy region bufora, a nie źródłowy obraz przed rasterem,
- zostawia znak komórki bez zmian,
- modyfikuje kolory `fg` i `bg`,
- wykonuje kwantyzację kolorów, lekkie wygładzanie, przyciemnianie krawędzi i opcjonalne nasycenie,
- obsługuje dwa tryby łączenia wyniku: `replace` i `overlay`.

## Jak działa

Kolejność przetwarzania w implementacji jest następująca:

1. pobranie snapshotu regionu bufora,
2. opcjonalne wygładzanie `simplify` w kilku przebiegach box-blur,
3. kwantyzacja kolorów przez `levels`,
4. wykrycie granic kolorów na sąsiednich komórkach,
5. przyciemnienie krawędzi przez `edge_strength` i `edge_width`,
6. przeskalowanie nasycenia przez `saturation`,
7. zapis wyniku z użyciem `replace` albo `overlay`.

Efekt działa na komórkach, więc jego wynik zależy od tego, co zostało już narysowane w buforze sceny.

## Parametry i walidacja

Aktualny schemat efektu dopuszcza:

- `levels`: liczba całkowita, zakres `1..100`, krok `1`
- `simplify`: liczba całkowita, zakres `0..8`, krok `1`
- `edge_fidelity`: liczba zmiennoprzecinkowa, zakres `0..1`
- `edge_strength`: liczba zmiennoprzecinkowa, zakres `0..1`
- `edge_width`: liczba całkowita, zakres `1..3`, krok `1`
- `saturation`: liczba zmiennoprzecinkowa, zakres `0.5..1.5`
- `blend_mode`: `replace` albo `overlay`

Jeśli edytor lub walidator pokazuje błąd w stylu `Value is not divisible by 0.05000000074505806`, oznacza to wymóg JSON Schema dla wartości zapisanej z krokiem `0.05`. Wtedy trzeba wpisać wartość będącą wielokrotnością `0.05`.

## Gdzie jest używany

W aktualnym repo `cutout` jest używany w:

- [mods/shell-quest/scenes/02-intro-boot/layers/main.yml](/home/ppotepa/git/shell-quest/mods/shell-quest/scenes/02-intro-boot/layers/main.yml)
- [mods/test-scenes/stages/door-cutscene.yml](/home/ppotepa/git/shell-quest/mods/test-scenes/stages/door-cutscene.yml)

W `02-intro-boot` efekt jest przypisany do sprite'a z GIF-em `cutscene.gif`. W `test-scenes` efekt jest używany w scenie testowej drzwi.

## Ograniczenia

- Efekt nie analizuje źródłowego PNG/GIF przed rasteryzacją.
- `progress` nie zmienia wyniku przetwarzania w samej implementacji efektu.
- Efekt nie zmienia znaku komórki, więc styl zależy od tego, jaki bufor wejściowy już istnieje.
- Parametry są walidowane przez schematy modów, więc błędy typu "property not allowed" zwykle oznaczają zły kontekst YAML, a nie problem samego efektu.

## Troubleshooting

- `Property stretch-to-area is not allowed.` oznacza, że klucz został użyty w miejscu, którego aktualny schemat YAML nie akceptuje. Trzeba sprawdzić, czy ten wpis jest w odpowiednim węźle `scene`, `layer`, `sprite`, `object` albo `stage`.
- `Value is not divisible by 0.05000000074505806.` oznacza, że wartość musi pasować do kroku `0.05`. Najprościej wpisać liczbę taką jak `0.05`, `0.10`, `0.15` itd.
- Jeśli efekt nie daje oczekiwanego wyglądu, sprawdź najpierw, czy w danym węźle rzeczywiście ustawiony jest `name: cutout`, a nie tylko podobnie nazwany efekt lub inny etap przetwarzania.


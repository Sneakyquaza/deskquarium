# Équilibrage (lot 1)

Toutes les valeurs sont dans `src/jeu.rs`. Méthode « tableur d'abord » (skill game-feel-balancing) : formules simples, puis simulation d'un joueur glouton pour vérifier les paliers (`aquarium --simulation`, 3 parties de 120 h de jeu).

## Revenu d'un poisson

`revenu/s = base(rareté) × taille × 1,2^(niveau − 1)`

| Rareté | Base /s | Couleur |
|---|---|---|
| Commun | 1 | gris #b4bcc4 |
| Peu commun | 4 | vert #6cd86a |
| Rare | 16 | bleu #4ca0ff |
| Épique | 70 | violet #b86cff |
| Légendaire | 320 | or #ffc845 |
| Mythique | 1 500 | rouge #ff526e |
| Céleste | 7 500 | turquoise #72f2e6 |
| Irréel | 40 000 | rose + arc-en-ciel animé |

Rapport ≈ ×4,5 par rareté (exponentielle douce). Taille : 0,6 à 1,5, tirée `0,6 + 0,9·u^1,5` (les petits sont plus fréquents, moyenne ≈ 0,96).

## Œufs

`prix = prix de base × (1 + n/10)²`, n = œufs de ce type déjà ouverts (polynomial : chaque œuf monte, sans mur exponentiel pour la chasse aux raretés hautes).

| Œuf | Prix de base | Commun | Peu c. | Rare | Épique | Légend. | Mythique | Céleste | Irréel |
|---|---|---|---|---|---|---|---|---|---|
| Rivière | 80 | 70 | 25 | 4,5 | 0,5 | | | | |
| Lagon | 5 000 | 30 | 42 | 20 | 6,5 | 1,5 | | | |
| Abysses | 1 000 000 | | 20 | 42 | 26 | 9,5 | 2,5 | | |
| Cosmique | 500 000 000 | | | 22 | 36 | 26 | 11 | 4,5 | 0,5 |

Tables vérifiées par test (200 000 tirages par œuf, écart < 0,35 point).

## Niveaux (« Nourrir »)

`coût(n → n+1) = base × taille × 120 × 1,9^(n − 1)`, niveau max 10 (×5,16).
Rentabilité d'un niveau = 600 s au niveau 1, ≈ 6 h 30 au niveau 9 : puits d'argent de long terme, l'œuf suivant reste le meilleur choix la plupart du temps.

## Autres règles

- Départ : un Commun de taille 1 offert → premier œuf (80) en **1 min 20 s**.
- Aquarium : 40 poissons qui nagent et rapportent ; au-delà, réserve (ne rapporte pas). Le réglage « Poissons dessinés » ne change que l'affichage.
- Relâcher : rend 20 s du revenu du poisson (jamais rentable de relâcher un œuf frais, sauf un Rare d'une rivière, voulu).
- Hors ligne : revenu × durée depuis la dernière sauvegarde, plafond 8 h, message « Pendant ton absence : +X ».
- Argent plafonné à 1e300 (le JSON n'accepte pas l'infini). Affichage : 742 · 1,23 k · 45,6 M · 1,2 Md · Bn · Bd · Tn · Td · Qn · Qd, puis 1,23e30.

## Simulation (joueur glouton, temps de fond d'écran allumé)

| Jalon | Partie 1 | Partie 2 | Partie 3 |
|---|---|---|---|
| 1er œuf de rivière | 1 min 20 | 1 min 20 | 1 min 20 |
| 1er œuf de lagon | 9 min | 9 min | 11 min |
| 1 k/s | 1 h 24 | 38 min | 1 h 14 |
| 1er œuf des abysses | 2 h 47 | 2 h 11 | 2 h 36 |
| 10 k/s | 11 h 28 | 6 h 57 | 7 h 50 |
| 1er œuf cosmique | 27 h | 22 h 52 | 21 h 53 |
| 1er Céleste | — | 37 h | — |
| Revenu après 120 h | 49 k/s | 202 k/s | 94 k/s |

Lecture : au rythme d'un PC allumé ~8 h par jour (+ 8 h hors ligne la nuit), les abysses arrivent le 1er jour, le cosmique vers le 2e-3e jour, Céleste et Irréel restent des chasses sur la semaine et au-delà. Un vrai joueur, moins optimal, sera plus lent : à revoir avec ses retours (leviers : prix des œufs, `revenu_base`, pente du prix `n/10`).

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
| Préhistorique (lot 3) | 220 000 | ambre #e89a3a + sédiments qui coulent |

Rapport ≈ ×4,5 par rareté (exponentielle douce). Taille : 0,6 à 1,5, tirée `0,6 + 0,9·u^1,5` (les petits sont plus fréquents, moyenne ≈ 0,96).

## Œufs

`prix = prix de base × (1 + n/10)²`, n = œufs de ce type déjà ouverts (polynomial : chaque œuf monte, sans mur exponentiel pour la chasse aux raretés hautes).

| Œuf | Prix de base | Commun | Peu c. | Rare | Épique | Légend. | Mythique | Céleste | Irréel | Préhist. |
|---|---|---|---|---|---|---|---|---|---|---|
| Rivière | 80 | 70 | 25 | 4,5 | 0,5 | | | | | |
| Lagon | 5 000 | 30 | 42 | 20 | 6,5 | 1,5 | | | | |
| Abysses | 1 000 000 | | 20 | 42 | 26 | 9,5 | 2,5 | | | |
| Cosmique | 500 000 000 | | | 22 | 36 | 26 | 11 | 4,5 | 0,5 | |
| Ambre (lot 3) | 30 000 000 000 | | | | | | | 12 | 8 | 80 |

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

## Espèces (lot 2)

L'espèce et la variante d'un poisson sont tirées de sa graine (`especes::choisir`), après la rareté : le flux aléatoire des œufs est le même qu'au lot 1, donc toutes les courbes ci-dessus restent valables. Le revenu ne dépend pas de l'espèce (pas de bonus de taille). Dans une rareté, chaque espèce a le poids 1, sauf Jörmungandr (0,35).

Affichage : la limite `max_affiches` (40) compte des places : créature énorme = 2, colossale = 4, deux colossales au plus. Tous les poissons de l'aquarium (40 au plus) rapportent, affichés ou non.

## Lot 3 : rareté Préhistorique et œuf d'ambre

- 9e rareté au-dessus d'Irréel : revenu de base 220 000/s (×5,5, le même pas que les autres). Ses 10 espèces (mégalodon, mosasaure, liopleurodon, ichthyosaure, dunkleosteus, hélicoprion, anomalocaris, ammonite, basilosaurus, trilobite) ne sortent que de l'œuf d'ambre.
- Œuf d'ambre : 30 Md (3e10), 80 % Préhistorique, 12 % Céleste, 8 % Irréel. Gain moyen ≈ 180 k/s par œuf : un peu moins rentable que le cosmique neuf, beaucoup plus que le cosmique déjà ouvert des centaines de fois → objectif de fin de partie.
- Sauvegardes : `oeufs_ouverts` et `eclos` sont des listes complétées par `migrer` (version 3) ; une sauvegarde v1/v2 se charge sans perte (test `sauvegarde_lot1_migree_sans_perte`).
- Simulation (600 h = 25 jours de fond d'écran, le joueur glouton n'économise pas plus d'un jour pour un œuf) : partie 1 et 3 plafonnent à 150-190 k/s sans atteindre l'ambre en 25 jours ; partie 2 ouvre le 1er œuf d'ambre à 11 j 6 h, passe à 1 M/s dans l'heure et finit à 39 M/s. L'ambre est donc une chasse de 2 à 4 semaines ; levier si trop long : son prix.

## v0.8 : shiny, modificateurs et index

Tirés à l'éclosion à partir de la graine du poisson (`Forme::tirer`, flux à part : le tirage des raretés et des espèces ne change pas). Shiny et modificateur sont indépendants (un poisson peut être shiny ET Galaxie : ×60). Au plus un modificateur. Le poisson offert au départ est toujours normal.

| Forme | Chance | Revenu | Effet |
|---|---|---|---|
| Shiny | 1/100 | ×5 | vraie recoloration (teinte dominante tournée de 0,36 à 0,7 tour selon l'espèce, gris teintés), traînée d'étincelles, étoiles, révélation fêtée |
| Doré | 1/20 | ×1,5 | rampe dorée à 40 %, reflet qui glisse, anneau doré à éclats |
| Diamant | 1/60 | ×2,5 | blanc bleuté à facettes, facettes qui s'allument sur l'anneau |
| Électrique | 1/120 | ×3 | couleurs vives un peu froides, anneau qui grésille, arcs brefs |
| Arc-en-ciel | 1/150 | ×4 | bandes de teinte qui tournent (palette tournée, pas chaque pixel) |
| Ombre | 1/250 | ×5 | plus sombre, violacée, +12 % (agrandie au plus proche, jamais réduite), fumée |
| Cristal | 1/400 | ×7 | pâle, translucide, facettes |
| Galaxie | 1/1000 | ×12 | ciel de nuit, étoiles qui clignent |

Espérance d'un poisson éclos : modificateurs ×1,129 (+12,9 %), shiny ×1,04, ensemble **×1,174** (`Forme::esperance`, testé). Le joueur glouton de la simulation compte cette espérance dans le gain d'un œuf ; « Équiper les meilleurs » et « Auto » suivent le revenu réel (forme comprise, test `equiper_les_meilleurs_avec_formes`). Le coût de « Nourrir » suit aussi la forme : un niveau se rentabilise pareil (600 s au niveau 1), shiny ou non.

**Index** (onglet Index de l'Écloserie) : Normal (espèce éclose sous n'importe quelle forme), un par modificateur, Shiny ; 9 index de 61 espèces. Chaque index complet donne **+5 % de tout le revenu, pour toujours** (`BONUS_INDEX`, multiplicatif sur le revenu de l'aquarium, gains hors ligne compris ; 9 index = +45 %). Les anciennes sauvegardes (v1-v3) chargent tous leurs poissons normaux, et chaque espèce déjà vue compte dans l'index Normal (test `sauvegarde_v3_migree_en_normal`).

Tirages vérifiés par test (2 millions : écart < 5 écarts-types par modificateur, shiny × Doré indépendants).

Simulation (600 h = 25 jours de fond d'écran, mêmes règles que le lot 3) :

| Jalon | Partie 1 | Partie 2 | Partie 3 |
|---|---|---|---|
| 1 k/s | 1 h 39 | 27 min | 2 h 05 |
| 1er œuf des abysses | 2 h 53 | 2 h 05 | 3 h 14 |
| 10 k/s | 10 h 03 | 7 h 43 | 7 h 05 |
| 1er œuf cosmique | 1 j 1 h | 1 j 4 h | 21 h 47 |
| 1er œuf d'ambre | — | 19 j 3 h | — |
| Fin (25 j) | 100 k/s | 41,1 M/s | 159 k/s |
| Shiny / modifiés dans l'aquarium | 2 / 7 | 1 / 6 | 0 / 7 |
| Index Normal | 35/61 | 51/61 | 36/61 |

Lecture : les premières heures bougent peu (l'espérance ne vaut que +17 %, et un modifié est rare au début) ; un shiny ou un Diamant tôt dans la partie se sent (×2,5 à ×5 sur un poisson). Aucun index n'est complet en 25 jours : l'index Normal (dernières espèces Préhistoriques et Irréelles) est l'objectif du mois, les index de modificateurs et Shiny sont des chasses de très long terme (environ 700 œufs ouverts en 25 jours → ~7 shiny et ~35 Dorés pour 61 espèces). Leviers si c'est trop lent : `CHANCE_SHINY`, chances des modificateurs, ou un index Normal par rareté.

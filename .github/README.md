# Hex for Android · Hex FR

[English](#english) · [Français](#français)

An unofficial Android fork of **Hex** for local, on-device dictation in French
and other languages. A floating bubble records; the speech engine transcribes
on the phone; the text goes into whatever field you are typing in. Nothing you
say leaves the device.

This repository is a fork of a fork. See [Credits](#credits--lineage), and
[Branches](#branches) for where the work lives.

## English

### What this is

[Hex](https://anoma.ly/hex) is a local dictation app for Mac, Windows and Linux
by Anomaly. [monkey-w1n5t0n](https://github.com/monkey-w1n5t0n/hex-for-android)
ported it to Android: a Kotlin app with no AndroidX and no third-party
dependencies, wrapping Hex's Rust speech engine (`transcribe.cpp`) over JNI.
This fork keeps that base and makes it usable in French.

### What we changed (branch `fr`)

| | |
|---|---|
| Speech model | **Parakeet TDT 0.6B v3** instead of the English-only Parakeet Unified English — 25 European languages, detected automatically, same engine and size |
| Dictated punctuation | French commands (*nouvelle ligne*, *virgule*, *point*, *ouvrez les guillemets*, *arobase*…) active only for French; the upstream English and Spanish rules are kept |
| Numbers | French number words become digits on the phone: *vingt-cinq* → `25`, *quatre-vingt-dix-huit* → `98`, *deux mille vingt-six* → `2026`, *trois pour cent* → `3 %`, *trois virgule quatorze* → `3,14`; a lone *un*/*une* and compounds like *vingt-cinquième* stay words |
| Interface | follows the phone's system language, French table with an English fallback |
| Build | a signed release APK (`0.3.1-fr`) and a GitHub Actions workflow that builds the Rust core twice and runs its tests on a clean runner |
| Contribution | branch `i18n` carries a clean, upstream-shaped version of the translation mechanism with French, Simplified Chinese and Serbian tables |

Install and build instructions: [`android/README.md`](../android/README.md).
Prebuilt APKs: [Releases](../../../releases).

### Branches

| Branch | Contents |
|---|---|
| `android` | mirror of the upstream Android port (default branch) |
| `fr` | the French build described above |
| `i18n` | clean branch based on upstream, for the translation PR |
| `patch-1` | the one-line fix to the Releases link |

### Credits / lineage

1. **[anomalyco/hex](https://github.com/anomalyco/hex)** — Hex for Mac, Windows
   and Linux, MIT, by Anomaly. Everything in this repository ultimately comes
   from there. This port is **not made or endorsed by Hex's authors**.
2. **[monkey-w1n5t0n/hex-for-android](https://github.com/monkey-w1n5t0n/hex-for-android)** —
   the unofficial Android port, MIT, and the repository this one is forked from.
   The floating bubble, the accessibility insertion, the JNI core and the whole
   `android/` tree are its work.
3. **This fork** — the French and multilingual additions above, MIT.

> If this fork is ever rebuilt directly from `anomalyco/hex`, the credit above
> stays: the Android layer came from `monkey-w1n5t0n/hex-for-android`, not from
> the desktop project.

Licences: code MIT; NVIDIA Parakeet CC-BY-4.0 (downloaded from Hugging Face at
first run); `transcribe.cpp` MIT; Inter and IBM Plex Mono SIL OFL 1.1.

## Français

### Ce que c'est

[Hex](https://anoma.ly/hex) est une application de dictée locale d'Anomaly pour
Mac, Windows et Linux.
[monkey-w1n5t0n](https://github.com/monkey-w1n5t0n/hex-for-android) l'a portée
sur Android : une application Kotlin, sans AndroidX ni dépendance tierce, qui
enveloppe le moteur Rust de Hex (`transcribe.cpp`) derrière JNI. Ce fork garde
cette base et la rend utilisable en français.

### Ce qu'on a fait (branche `fr`)

| | |
|---|---|
| Modèle vocal | **Parakeet TDT 0.6B v3** à la place du modèle anglais Parakeet Unified English — 25 langues européennes, détectées automatiquement, même moteur et même taille |
| Ponctuation dictée | commandes françaises (*nouvelle ligne*, *virgule*, *point*, *ouvrez les guillemets*, *arobase*…) actives seulement en français ; les règles anglaises et espagnoles d'origine sont conservées |
| Nombres | les nombres dits en français s'écrivent en chiffres sur le téléphone : *vingt-cinq* → `25`, *quatre-vingt-dix-huit* → `98`, *deux mille vingt-six* → `2026`, *trois pour cent* → `3 %`, *trois virgule quatorze* → `3,14` ; un *un*/*une* isolé et les composés comme *vingt-cinquième* restent en mots |
| Interface | suit la langue du téléphone, table française avec repli sur l'anglais |
| Build | un APK release signé (`0.3.1-fr`) et un workflow GitHub Actions qui compile le cœur Rust deux fois et lance ses tests sur une machine propre |
| Contribution | la branche `i18n` porte une version propre, prête pour l'amont, du mécanisme de traduction avec les tables français, chinois simplifié et serbe |

Installation et compilation : [`android/README.md`](../android/README.md).
APK prêts à installer : [Releases](../../../releases).

### Branches

| Branche | Contenu |
|---|---|
| `android` | copie du portage Android amont (branche par défaut) |
| `fr` | la version française décrite ci-dessus |
| `i18n` | branche propre basée sur l'amont, pour la PR de traduction |
| `patch-1` | le correctif d'une ligne du lien Releases |

### Crédits / filiation

1. **[anomalyco/hex](https://github.com/anomalyco/hex)** — Hex pour Mac, Windows
   et Linux, MIT, par Anomaly. Tout ce dépôt en vient finalement. Ce portage
   **n'est ni réalisé ni endossé par les auteurs de Hex**.
2. **[monkey-w1n5t0n/hex-for-android](https://github.com/monkey-w1n5t0n/hex-for-android)** —
   le portage Android non officiel, MIT, et le dépôt dont celui-ci est le fork.
   La bulle flottante, l'insertion par accessibilité, le cœur JNI et tout
   l'arbre `android/` sont son travail.
3. **Ce fork** — les ajouts français et multilingues ci-dessus, MIT.

> Si ce fork est un jour reforgé directement depuis `anomalyco/hex`, ce crédit
> reste : la couche Android vient de `monkey-w1n5t0n/hex-for-android`, pas du
> projet desktop.

Licences : code MIT ; NVIDIA Parakeet CC-BY-4.0 (téléchargé depuis Hugging Face
au premier lancement) ; `transcribe.cpp` MIT ; Inter et IBM Plex Mono SIL OFL 1.1.

# Instalátor Spyker Tým

Instalátor češtin do her.
[Web](https://spykertym.cz)

## Určeno pro
- Windows 11
- Linux x86_64

## Prerekvizity

- [Node.js](https://nodejs.org/en/download)
- [Rust](https://rust-lang.org/tools/install/)
- [Xdelta3](https://github.com/jmacd/xdelta) (pouze pro Linux, Windows má exe přibalené do instalačky)

## Instalace
```bash
git clone https://github.com/Jarda-H/spykertym-installer.git
cd spykertym-installer
npm i
```
nebo [Releases](https://github.com/Jarda-H/spykertym-installer/releases) (pouze Windows)

## Jak spustit

### Dev verze

```bash
npm run d
```

### Build

```bash
npm run b
```

## Problémy

Při problémech nám napište na jakýkoliv komunikační kanál, popř. přímo zde na Githubu založte Issue popisující kroky a nejlépe čas, kdy problém nastal.

## Pomoc s vývojem

Pokud máte nápad jak instalátor vylepšit, vytvořte si fork tohoto rep. a vytvořte pull request s Vaší změnou! Nebo nám napište a my se pokusíme funkci implementovat (pokud to uznáme za vhodné).
# mm
Un semplice CLI scritto in Rust per salvare ed eseguire "shortcut" (alias) per comandi da terminale.

## Cos'è

`mm` ti permette di associare un nome breve (alias) a un comando (o sequenza di comandi) più lungo, 
e di richiamarlo in seguito senza doverlo riscrivere.

Gli shortcut vengono salvati in un file JSON:
- **locale**: `.mm` nella cartella corrente
- **globale**: `~/.mm_global`

## Installazione

```bash
cargo build --release
```

L'eseguibile sarà disponibile in `target/release/mm`.

## Utilizzo

### Aggiungere uno shortcut

```bash
mm add <alias> <comando...>
```

Esempio:

```bash
mm add build "cargo build --release"
```

### Eseguire uno shortcut

```bash
mm <alias>
```

Esempio:

```bash
mm build
```

### Mostrare il comando associato a uno shortcut

```bash
mm show <alias>
```

### Elencare tutti gli shortcut

```bash
mm list
```

### Rimuovere uno shortcut

```bash
mm remove <alias>
```

### Usare il file di configurazione globale

Aggiungi `-g` / `--global` a qualsiasi comando per operare su `~/.mm_global` invece che sul file `.mm` locale:

```bash
mm --global add deploy ./deploy.sh
mm --global deploy
```

## Struttura del progetto
- `main.rs` – entry point, dispatch dei comandi
- `cli.rs` – definizione degli argomenti CLI (tramite `clap`)
- `commands.rs` – logica di add/remove/show/list/execute
- `utils.rs` – funzioni di supporto (espansione path, risoluzione config)

## Note
- Gli shortcut vengono eseguiti tramite `sh -c`, quindi supportano pipe, redirect, ecc.
- Se il file di configurazione non esiste, viene creato automaticamente al primo utilizzo.
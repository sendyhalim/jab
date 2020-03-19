# Jab
`jab` is a database state management tool, think of it as git but for database. You can commit your current db state and checkout to your previous db state.

[![Crates.io](https://img.shields.io/crates/v/jab)](https://crates.io/crates/jab)
[![Crates.io](https://img.shields.io/crates/l/jab)](LICENSE)

## 📠 Notes
* Currently only supports postgres.
* This project only works if you have `ON DELETE CASCADE` on every FK constraints otherwise we can't do clean restore (hopefully this will change in the future).

## 🔩 Installation
### Cargo
```bash
cargo install jab
```

### Manual (requires rust)
```bash
git clone git@github.com:sendyhalim/jab.git

make install
```

### Download
Dynamically linked binaries are only available for macos and linux. Go [here](https://github.com/sendyhalim/jab/releases/latest).

## 🎮 Usage
```bash
# First create project.
# Currently only supports postgres.
# ------------------------------------------
jab project create awesomestuff --database-uri="username:password@localhost:5433"

# Start commiting your db
# ------------------------------------------
jab project commit awesomestuff --message "my first commit"

# See log
# ------------------------------------------
jab project log awesomestuff

# Restore your db to the latest commit
# ------------------------------------------
jab project restore awesomestuff

# Restore your db to a specific commit
# ------------------------------------------
jab project restore awesomestuff [optional-hash]


# List of available projects
# ------------------------------------------
jab project list
```

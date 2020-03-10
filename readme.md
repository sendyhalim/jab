# Cid
CID is a database state management tool, think of it as git but for database. You can commit your current db state and checkout to your previous db state.

## Notes
* Currently only supports postgres.
* This project only works if you have `ON DELETE CASCADE` on every FK constraints otherwise we can't do clean restore (hopefully this will change in the future).

## Installation
### Manual (requires rust)
```bash
git clone git@github.com:sendyhalim/cid.git

make install
```

### Download
Dynamically linked binaries are only available for macos and linux. Go [here](https://github.com/sendyhalim/cid/releases/tag/0.0.1).

### Usage
```bash
# First create project.
# Currently only supports postgres.
cid project create awesomestuff --database-uri="username:password@localhost:5433"

# Start commiting your db
cid project commit awesomestuff --message "my first commit"

# See log
cid project log awesomestuff

# Restore your db
cid project restore awesomestuff <hash>

# List of available projects
cid project list
```

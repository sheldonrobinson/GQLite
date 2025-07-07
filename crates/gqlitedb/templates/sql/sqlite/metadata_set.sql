INSERT INTO gqlite_metadata (name, value) VALUES (:name, :value) ON CONFLICT(name) DO UPDATE SET value=:value

CREATE TABLE results (
    id          INTEGER PRIMARY KEY,
    -- unix seconds
    finished_at INTEGER NOT NULL,
    mode        TEXT    NOT NULL,
    mode_value  INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    wpm         REAL    NOT NULL,
    raw_wpm     REAL    NOT NULL,
    accuracy    REAL    NOT NULL
);

CREATE INDEX results_by_mode ON results (mode, mode_value, wpm);

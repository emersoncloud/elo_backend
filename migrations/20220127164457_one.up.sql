-- Add up migration script here
create table matches (
    label VARCHAR(4) NOT NULL,
    PRIMARY KEY (label)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

create table players (
    id INT NOT NULL AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    elo INT NOT NULL,
    picture VARCHAR(255),
    PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

create table free_players (
    match_label VARCHAR(4) NOT NULL,
    player_id INT NOT NULL,
    PRIMARY KEY (match_label, player_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

create table team_1 (
    match_label VARCHAR(4) NOT NULL,
    player_id INT NOT NULL,
    PRIMARY KEY (match_label, player_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

create table team_2 (
    match_label VARCHAR(4) NOT NULL,
    player_id INT NOT NULL,
    PRIMARY KEY (match_label, player_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

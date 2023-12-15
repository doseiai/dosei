CREATE TABLE cron_jobs (
   uuid UUID NOT NULL,
   schedule TEXT NOT NULL,
   entrypoint TEXT NOT NULL,
   owner_id UUID NOT NULL,
   deployment_id TEXT NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   PRIMARY KEY (uuid),
   UNIQUE (uuid, owner_id, deployment_id)
);

CREATE TABLE envs (
   uuid UUID NOT NULL,
   name TEXT NOT NULL,
   value integer[] NOT NULL,
   project_id UUID NOT NULL,
   owner_id UUID NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   PRIMARY KEY (uuid),
   UNIQUE (name, project_id, owner_id)
);

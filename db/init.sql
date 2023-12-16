CREATE TABLE cron_jobs (
   id UUID NOT NULL,
   schedule TEXT NOT NULL,
   entrypoint TEXT NOT NULL,
   owner_id UUID NOT NULL,
   deployment_id TEXT NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   PRIMARY KEY (id),
   UNIQUE (id, owner_id, deployment_id)
);

CREATE TABLE envs (
   id UUID NOT NULL,
   name TEXT NOT NULL,
   value integer[] NOT NULL,
   project_id UUID NOT NULL,
   owner_id UUID NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   PRIMARY KEY (id),
   UNIQUE (name, project_id, owner_id)
);

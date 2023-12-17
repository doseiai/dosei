CREATE TABLE cron_jobs (
   id UUID NOT NULL,
   schedule TEXT NOT NULL,
   entrypoint TEXT NOT NULL,
   owner_id UUID NOT NULL,
   --- Git Commit (sha1 hash)
   deployment_id TEXT NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   PRIMARY KEY (id)
);

CREATE TABLE envs (
   id UUID NOT NULL,
   name TEXT NOT NULL,
   value TEXT NOT NULL,
   -- project id can be nullable
   project_id UUID NULL,
   user_id UUID NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   PRIMARY KEY (id),
   UNIQUE (name, project_id, user_id)
);

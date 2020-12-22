CREATE TABLE invitations (
  id UUID PRIMARY KEY,
  email VARCHAR NOT NULL,
  expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
  forgot_pw BOOL NOT NULL
);

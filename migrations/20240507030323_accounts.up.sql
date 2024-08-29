-- Add up migration script here
CREATE TABLE bw_account (
    id BIGINT PRIMARY KEY DEFAULT next_id(),
    name VARCHAR (255) NOT NULL,
    email VARCHAR (255) UNIQUE NOT NULL,
    password VARCHAR (255) NOT NULL,

    status account_status NOT NULL DEFAULT 'inactive',
    language language NOT NULL DEFAULT 'en-US',

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT NULL,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TRIGGER update_bw_account_updated_at
BEFORE UPDATE ON bw_account
FOR EACH ROW 
EXECUTE FUNCTION update_at();

CREATE INDEX idx_bw_account_name ON bw_account (name);
CREATE INDEX idx_bw_account_email ON bw_account (email);

COMMENT ON COLUMN bw_account.id IS '用户ID';
COMMENT ON COLUMN bw_account.NAME IS '用户名';
COMMENT ON COLUMN bw_account.email IS '用户邮箱';
COMMENT ON COLUMN bw_account.password IS '用户密码';
COMMENT ON COLUMN bw_account.language IS '用户系统语言设置';
COMMENT ON COLUMN bw_account.created_at IS '记录创建时间';
COMMENT ON COLUMN bw_account.updated_at IS '记录更新时间';
COMMENT ON COLUMN bw_account.deleted_at IS '记录删除时间';

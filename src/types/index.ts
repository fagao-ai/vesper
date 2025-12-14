export interface SSHConnection {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  auth_method: 'password' | 'key';
  password?: string;
  key_path?: string;
  status: 'disconnected' | 'connecting' | 'connected' | 'error';
  last_connected?: string | Date;
}

export interface SSHTunnel {
  id: string;
  name: string;
  connection_id: string;
  tunnel_type: 'local' | 'remote';
  local_port: number;
  remote_host: string;
  remote_port: number;
  status: 'inactive' | 'active' | 'error';
  auto_reconnect: boolean;
}

export interface AppConfig {
  theme: 'light' | 'dark' | 'auto';
  language: string;
  autoStart: boolean;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
  defaultKeyPath: string;
}
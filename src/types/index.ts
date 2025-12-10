export interface SSHConnection {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  authMethod: 'password' | 'key';
  password?: string;
  keyPath?: string;
  status: 'disconnected' | 'connecting' | 'connected' | 'error';
  lastConnected?: Date;
}

export interface SSHTunnel {
  id: string;
  name: string;
  connectionId: string;
  type: 'local' | 'remote' | 'dynamic';
  localPort: number;
  remoteHost: string;
  remotePort: number;
  status: 'inactive' | 'active' | 'error';
  autoReconnect: boolean;
}

export interface AppConfig {
  theme: 'light' | 'dark' | 'auto';
  language: string;
  autoStart: boolean;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
  defaultKeyPath: string;
}
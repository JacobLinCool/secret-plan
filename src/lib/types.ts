// Define the breach state enum to match Rust's BreachState
export enum BreachState {
	Unknown = 0,
	Safe = 1,
	Compromised = 2
}

// Credential type definition matching Rust model
export interface Credential {
	uuid: string;
	site: string;
	username: string;
	secret_enc: string;
	tags: string[];
	created_at: number;
	updated_at: number;
	expires_at: number | null;
	strength: number;
	breach_state: BreachState;
}

// Secret type definition matching Rust model
export interface Secret {
	password: string;
	notes?: string;
	totp?: string;
	custom_fields: Record<string, string>;
}

// App settings type definition
export interface AppSettings {
	theme: 'light' | 'dark' | 'system';
	auto_lock_timeout: number; // minutes, 0 = never
	password_gen_defaults: {
		length: number;
		use_uppercase: boolean;
		use_lowercase: boolean;
		use_numbers: boolean;
		use_symbols: boolean;
		exclude_similar: boolean;
	};
}

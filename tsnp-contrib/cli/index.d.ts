// TypeScript type definitions for CLI utilities

declare module "tsnp-cli" {
    /**
     * Get command-line argument count
     * @returns Number of arguments
     */
    export function argc(): number;
    
    /**
     * Get command-line argument by index
     * @param index - Argument index (0 = program name)
     * @returns Argument value
     */
    export function argv(index: number): string;
    
    /**
     * Get current timestamp in milliseconds
     * @returns Unix timestamp in milliseconds
     */
    export function now_ms(): number;
    
    /**
     * Sleep for specified milliseconds
     * @param ms - Milliseconds to sleep
     */
    export function sleep(ms: number): void;
    
    /**
     * Get environment variable
     * @param name - Variable name
     * @returns Variable value or empty string
     */
    export function getenv(name: string): string;
    
    /**
     * Exit program with code
     * @param code - Exit code
     */
    export function exit(code: number): void;
}
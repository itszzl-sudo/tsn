// TypeScript type definitions for file system

declare module "tsnp-fs" {
    /**
     * Write string to file (overwrite)
     * @param path - File path
     * @param content - Content to write
     */
    export function file_write(path: string, content: string): void;
    
    /**
     * Append string to file
     * @param path - File path
     * @param content - Content to append
     */
    export function file_append(path: string, content: string): void;
    
    /**
     * Read file content
     * @param path - File path
     * @returns File content as string
     */
    export function file_read(path: string): string;
    
    /**
     * Check if file exists
     * @param path - File path
     * @returns 1 if exists, 0 otherwise
     */
    export function file_exists(path: string): number;
    
    /**
     * Get file size in bytes
     * @param path - File path
     * @returns File size in bytes, -1 if not exists
     */
    export function file_size(path: string): number;
}
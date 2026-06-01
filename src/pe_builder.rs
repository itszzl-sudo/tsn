use anyhow::Result;

pub struct PeBuilder {
    code: Vec<u8>,
    data: Vec<u8>,
}

impl PeBuilder {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            data: Vec::new(),
        }
    }
    
    pub fn add_code(&mut self, code: &[u8]) {
        self.code.extend_from_slice(code);
    }
    
    pub fn add_data(&mut self, data: &[u8]) -> usize {
        let offset = self.data.len();
        self.data.extend_from_slice(data);
        offset
    }
    
    pub fn build(&self) -> Result<Vec<u8>> {
        let mut pe = Vec::new();
        
        // DOS Header (64 bytes)
        pe.extend_from_slice(&self.build_dos_header());
        
        // DOS stub "This program cannot be run in DOS mode"
        pe.extend_from_slice(b"This program cannot be run in DOS mode.\r\n$");
        while pe.len() % 8 != 0 {
            pe.push(0);
        }
        
        // PE Signature
        pe.extend_from_slice(b"PE\0\0");
        
        // COFF File Header (20 bytes)
        pe.extend_from_slice(&self.build_coff_header());
        
        // Optional Header (for PE32+)
        pe.extend_from_slice(&self.build_optional_header());
        
        // Section Headers
        pe.extend_from_slice(&self.build_section_headers());
        
        // Code Section
        while pe.len() % 0x1000 != 0 {
            pe.push(0);
        }
        pe.extend_from_slice(&self.code);
        while pe.len() % 0x1000 != 0 {
            pe.push(0);
        }
        
        // Data Section
        pe.extend_from_slice(&self.data);
        while pe.len() % 0x1000 != 0 {
            pe.push(0);
        }
        
        Ok(pe)
    }
    
    fn build_dos_header(&self) -> [u8; 64] {
        let mut dos_header = [0u8; 64];
        
        // MZ signature
        dos_header[0] = 0x4D; // 'M'
        dos_header[1] = 0x5A; // 'Z'
        
        // Bytes on last page
        dos_header[2] = 0x90;
        dos_header[3] = 0x00;
        
        // Pages in file
        dos_header[4] = 0x03;
        dos_header[5] = 0x00;
        
        // Relocations
        dos_header[6] = 0x00;
        dos_header[7] = 0x00;
        
        // Header size in paragraphs
        dos_header[8] = 0x04;
        dos_header[9] = 0x00;
        
        // Minimum extra paragraphs
        dos_header[10] = 0x00;
        dos_header[11] = 0x00;
        
        // Maximum extra paragraphs
        dos_header[12] = 0xFF;
        dos_header[13] = 0xFF;
        
        // Initial SS
        dos_header[14] = 0x00;
        dos_header[15] = 0x00;
        
        // Initial SP
        dos_header[16] = 0x00;
        dos_header[17] = 0x00;
        
        // Checksum
        dos_header[18] = 0x00;
        dos_header[19] = 0x00;
        
        // Initial IP
        dos_header[20] = 0x00;
        dos_header[21] = 0x00;
        
        // Initial CS
        dos_header[22] = 0x00;
        dos_header[23] = 0x00;
        
        // File address of relocation table
        dos_header[24] = 0x40;
        dos_header[25] = 0x00;
        
        // Overlay number
        dos_header[26] = 0x00;
        dos_header[27] = 0x00;
        
        // OEM identifier
        dos_header[36] = 0x00;
        dos_header[37] = 0x00;
        
        // OEM information
        dos_header[38] = 0x00;
        dos_header[39] = 0x00;
        
        // File address of PE header (at offset 0x40)
        let pe_offset = 0x80u32;
        dos_header[60] = (pe_offset & 0xFF) as u8;
        dos_header[61] = ((pe_offset >> 8) & 0xFF) as u8;
        dos_header[62] = ((pe_offset >> 16) & 0xFF) as u8;
        dos_header[63] = ((pe_offset >> 24) & 0xFF) as u8;
        
        dos_header
    }
    
    fn build_coff_header(&self) -> [u8; 20] {
        let mut coff = [0u8; 20];
        
        // Machine: AMD64
        coff[0] = 0x64;
        coff[1] = 0x86;
        
        // Number of sections
        coff[2] = 0x02; // .text and .data
        coff[3] = 0x00;
        
        // Time date stamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        coff[4] = (timestamp & 0xFF) as u8;
        coff[5] = ((timestamp >> 8) & 0xFF) as u8;
        coff[6] = ((timestamp >> 16) & 0xFF) as u8;
        coff[7] = ((timestamp >> 24) & 0xFF) as u8;
        
        // Pointer to symbol table (0 = none)
        coff[8] = 0x00;
        coff[9] = 0x00;
        coff[10] = 0x00;
        coff[11] = 0x00;
        
        // Number of symbols
        coff[12] = 0x00;
        coff[13] = 0x00;
        coff[14] = 0x00;
        coff[15] = 0x00;
        
        // Size of optional header
        coff[16] = 0xF0; // 240 bytes for PE32+
        coff[17] = 0x00;
        
        // Characteristics: executable
        coff[18] = 0x22;
        coff[19] = 0x00;
        
        coff
    }
    
    fn build_optional_header(&self) -> Vec<u8> {
        let mut opt = vec![0u8; 240];
        
        // Magic: PE32+ (0x20b)
        opt[0] = 0x0B;
        opt[1] = 0x02;
        
        // Linker version
        opt[2] = 0x0E; // Major
        opt[3] = 0x1C; // Minor
        
        // Size of code
        let code_size = self.code.len() as u32;
        opt[4] = (code_size & 0xFF) as u8;
        opt[5] = ((code_size >> 8) & 0xFF) as u8;
        opt[6] = ((code_size >> 16) & 0xFF) as u8;
        opt[7] = ((code_size >> 24) & 0xFF) as u8;
        
        // Size of initialized data
        let data_size = self.data.len() as u32;
        opt[8] = (data_size & 0xFF) as u8;
        opt[9] = ((data_size >> 8) & 0xFF) as u8;
        opt[10] = ((data_size >> 16) & 0xFF) as u8;
        opt[11] = ((data_size >> 24) & 0xFF) as u8;
        
        // Size of uninitialized data
        opt[12] = 0x00;
        opt[13] = 0x00;
        opt[14] = 0x00;
        opt[15] = 0x00;
        
        // Address of entry point
        opt[16] = 0x00;
        opt[17] = 0x10;
        opt[18] = 0x00;
        opt[19] = 0x00;
        
        // Base of code
        opt[20] = 0x00;
        opt[21] = 0x10;
        opt[22] = 0x00;
        opt[23] = 0x00;
        
        // Image base (PE32+)
        let image_base: u64 = 0x140000000;
        opt[24] = (image_base & 0xFF) as u8;
        opt[25] = ((image_base >> 8) & 0xFF) as u8;
        opt[26] = ((image_base >> 16) & 0xFF) as u8;
        opt[27] = ((image_base >> 24) & 0xFF) as u8;
        opt[28] = ((image_base >> 32) & 0xFF) as u8;
        opt[29] = ((image_base >> 40) & 0xFF) as u8;
        opt[30] = ((image_base >> 48) & 0xFF) as u8;
        opt[31] = ((image_base >> 56) & 0xFF) as u8;
        
        // Section alignment (4KB)
        opt[32] = 0x00;
        opt[33] = 0x10;
        opt[34] = 0x00;
        opt[35] = 0x00;
        
        // File alignment (512 bytes)
        opt[36] = 0x00;
        opt[37] = 0x02;
        opt[38] = 0x00;
        opt[39] = 0x00;
        
        // Operating system version
        opt[40] = 0x06; // Major
        opt[41] = 0x00;
        opt[42] = 0x00; // Minor
        opt[43] = 0x00;
        
        // Image version
        opt[44] = 0x00;
        opt[45] = 0x00;
        opt[46] = 0x00;
        opt[47] = 0x00;
        
        // Subsystem version
        opt[48] = 0x06; // Major
        opt[49] = 0x00;
        opt[50] = 0x00; // Minor
        opt[51] = 0x00;
        
        // Win32 version value
        opt[52] = 0x00;
        opt[53] = 0x00;
        opt[54] = 0x00;
        opt[55] = 0x00;
        
        // Size of image
        let image_size = 0x3000u32; // 3 pages
        opt[56] = (image_size & 0xFF) as u8;
        opt[57] = ((image_size >> 8) & 0xFF) as u8;
        opt[58] = ((image_size >> 16) & 0xFF) as u8;
        opt[59] = ((image_size >> 24) & 0xFF) as u8;
        
        // Size of headers
        opt[60] = 0x00;
        opt[61] = 0x01;
        opt[62] = 0x00;
        opt[63] = 0x00;
        
        // Checksum
        opt[64] = 0x00;
        opt[65] = 0x00;
        opt[66] = 0x00;
        opt[67] = 0x00;
        
        // Subsystem: Console
        opt[68] = 0x03;
        opt[69] = 0x00;
        
        // DLL characteristics
        opt[70] = 0x00;
        opt[71] = 0x40;
        
        // Size of stack reserve
        let stack_reserve: u64 = 0x100000;
        opt[72] = (stack_reserve & 0xFF) as u8;
        opt[73] = ((stack_reserve >> 8) & 0xFF) as u8;
        opt[74] = ((stack_reserve >> 16) & 0xFF) as u8;
        opt[75] = ((stack_reserve >> 24) & 0xFF) as u8;
        opt[76] = ((stack_reserve >> 32) & 0xFF) as u8;
        opt[77] = ((stack_reserve >> 40) & 0xFF) as u8;
        opt[78] = ((stack_reserve >> 48) & 0xFF) as u8;
        opt[79] = ((stack_reserve >> 56) & 0xFF) as u8;
        
        // Size of stack commit
        let stack_commit: u64 = 0x1000;
        opt[80] = (stack_commit & 0xFF) as u8;
        opt[81] = ((stack_commit >> 8) & 0xFF) as u8;
        opt[82] = ((stack_commit >> 16) & 0xFF) as u8;
        opt[83] = ((stack_commit >> 24) & 0xFF) as u8;
        opt[84] = ((stack_commit >> 32) & 0xFF) as u8;
        opt[85] = ((stack_commit >> 40) & 0xFF) as u8;
        opt[86] = ((stack_commit >> 48) & 0xFF) as u8;
        opt[87] = ((stack_commit >> 56) & 0xFF) as u8;
        
        // Size of heap reserve
        opt[88] = 0x00;
        opt[89] = 0x00;
        opt[90] = 0x00;
        opt[91] = 0x00;
        opt[92] = 0x00;
        opt[93] = 0x00;
        opt[94] = 0x00;
        opt[95] = 0x00;
        
        // Size of heap commit
        opt[96] = 0x00;
        opt[97] = 0x00;
        opt[98] = 0x00;
        opt[99] = 0x00;
        opt[100] = 0x00;
        opt[101] = 0x00;
        opt[102] = 0x00;
        opt[103] = 0x00;
        
        // Number of data directories
        opt[104] = 0x10;
        opt[105] = 0x00;
        opt[106] = 0x00;
        opt[107] = 0x00;
        
        opt
    }
    
    fn build_section_headers(&self) -> Vec<u8> {
        let mut sections = Vec::new();
        
        // .text section
        let mut text_section = vec![0u8; 40];
        
        // Name: .text
        text_section[0] = b'.';
        text_section[1] = b't';
        text_section[2] = b'e';
        text_section[3] = b'x';
        text_section[4] = b't';
        
        // Virtual size
        let code_size = self.code.len() as u32;
        text_section[8] = (code_size & 0xFF) as u8;
        text_section[9] = ((code_size >> 8) & 0xFF) as u8;
        text_section[10] = ((code_size >> 16) & 0xFF) as u8;
        text_section[11] = ((code_size >> 24) & 0xFF) as u8;
        
        // Virtual address
        text_section[12] = 0x00;
        text_section[13] = 0x10;
        text_section[14] = 0x00;
        text_section[15] = 0x00;
        
        // Size of raw data
        text_section[16] = (code_size & 0xFF) as u8;
        text_section[17] = ((code_size >> 8) & 0xFF) as u8;
        text_section[18] = ((code_size >> 16) & 0xFF) as u8;
        text_section[19] = ((code_size >> 24) & 0xFF) as u8;
        
        // Pointer to raw data
        text_section[20] = 0x00;
        text_section[21] = 0x01;
        text_section[22] = 0x00;
        text_section[23] = 0x00;
        
        // Characteristics: code | execute | read
        text_section[36] = 0x60;
        text_section[37] = 0x00;
        text_section[38] = 0x00;
        text_section[39] = 0x20;
        
        sections.extend_from_slice(&text_section);
        
        // .data section
        let mut data_section = vec![0u8; 40];
        
        // Name: .data
        data_section[0] = b'.';
        data_section[1] = b'd';
        data_section[2] = b'a';
        data_section[3] = b't';
        data_section[4] = b'a';
        
        // Virtual size
        let data_size = self.data.len() as u32;
        data_section[8] = (data_size & 0xFF) as u8;
        data_section[9] = ((data_size >> 8) & 0xFF) as u8;
        data_section[10] = ((data_size >> 16) & 0xFF) as u8;
        data_section[11] = ((data_size >> 24) & 0xFF) as u8;
        
        // Virtual address
        data_section[12] = 0x00;
        data_section[13] = 0x20;
        data_section[14] = 0x00;
        data_section[15] = 0x00;
        
        // Size of raw data
        data_section[16] = (data_size & 0xFF) as u8;
        data_section[17] = ((data_size >> 8) & 0xFF) as u8;
        data_section[18] = ((data_size >> 16) & 0xFF) as u8;
        data_section[19] = ((data_size >> 24) & 0xFF) as u8;
        
        // Pointer to raw data
        let data_offset = 0x1000 + (code_size / 0x1000 + 1) * 0x1000;
        data_section[20] = (data_offset & 0xFF) as u8;
        data_section[21] = ((data_offset >> 8) & 0xFF) as u8;
        data_section[22] = ((data_offset >> 16) & 0xFF) as u8;
        data_section[23] = ((data_offset >> 24) & 0xFF) as u8;
        
        // Characteristics: initialized data | read | write
        data_section[36] = 0xC0;
        data_section[37] = 0x00;
        data_section[38] = 0x00;
        data_section[39] = 0x40;
        
        sections.extend_from_slice(&data_section);
        
        sections
    }
}

impl Default for PeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

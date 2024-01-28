use anchor_lang::prelude::*;

#[account]
pub struct MetadataInfo {
    // This field could optionally be derived somehow too
    pub update_authority: Pubkey,
    /// This field needs to be derivable somehow
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub description: String,
    pub num_transfers: u32,
}

pub const MAX_URI_LEN: usize = 98;

fn format_uri(num: u32) -> String {
    let str = format!("https://bafybeiffh25vb32ns6zspqjxcpkvqzvgmdn6xrzwnnt7eghfqkwdiwpeaq.ipfs.nftstorage.link/{}.json", num);
    let suffix = "#".repeat(MAX_URI_LEN - str.len());
    format!("{}{}", &str, &suffix)
}

fn format_name(num: u32) -> String {
    format!("#{:4}", num)
}

impl MetadataInfo {
    pub fn init(self: &mut Self, mint: &Pubkey, update_authority: &Pubkey) {
        self.update_authority = *update_authority;
        self.mint = *mint;

        self.name = format_name(0);
        self.symbol = "UNDEAD".to_string();
        self.uri = format_uri(0);
        self.description = "This NFT changes its URI on every transfer".to_string();
        self.num_transfers = 0;
    }

    pub fn update_on_transfer(self: &mut Self) {
        self.num_transfers += 1;
        // Floor by 10000 to get to a new collection
        self.num_transfers = self.num_transfers % 10000;

        // Undead collection
        self.name = format_name(self.num_transfers);
        self.uri = format_uri(self.num_transfers);
    }
}

use scrypto::prelude::*;

blueprint! {
    struct Radiswap {
        /// The resource definition of LP token.
        lp_resource_def: ResourceDef,
        /// LP tokens mint badge.
        lp_mint_badge: Vault,
        /// The reserve for token A.
        a_pool: Vault,
        /// The reserve for token B.
        b_pool: Vault,
        /// The fee to apply for every swap
        fee: Decimal,
        /// The standard (Uniswap-like) DEX follows the X*Y=K rule. Since we enable a user defined 'lp_initial_supply', we need to store this value to recover incase all liquidity is removed from the system.
        /// Adding and removing liquidity does not change this ratio, this ratio is only changed upon swaps.
        lp_per_asset_ratio: Decimal,
    }

    impl Radiswap {
        /// Creates a Radiswap component for token pair A/B and returns the component address
        /// along with the initial LP tokens.
        pub fn new(
            a_tokens: Bucket,
            b_tokens: Bucket,
            lp_initial_supply: Decimal,
            lp_symbol: String,
            lp_name: String,
            lp_url: String,
            fee: Decimal,
        ) -> (Component, Bucket) {
            // Check arguments
            assert!(
                !a_tokens.is_empty() && !b_tokens.is_empty(),
                "You must pass in an initial supply of each token"
            );
            assert!(
                fee >= 0.into() && fee <= 1.into(),
                "Invalid fee in thousandths"
            );

            // Instantiate our LP token and mint an initial supply of them
            let lp_mint_badge = ResourceBuilder::new_fungible(18)
                .metadata("name", "LP Token Mint Auth")
                .flags(FREELY_TRANSFERABLE | FREELY_BURNABLE)
                .initial_supply_fungible(1);
            let lp_resource_def = ResourceBuilder::new_fungible(0)
                .metadata("symbol", lp_symbol)
                .metadata("name", lp_name)
                .metadata("url", lp_url)
                .flags(FREELY_TRANSFERABLE | MINTABLE | BURNABLE)
                .badge(lp_mint_badge.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();
            let lp_tokens = lp_resource_def.mint(lp_initial_supply, lp_mint_badge.present());

            // ratio = initial supply / (x * y) = initial supply / k
            let lp_per_asset_ratio = lp_initial_supply / (a_tokens.amount() * b_tokens.amount());

            // Instantiate our Radiswap component
            let radiswap = Self {
                lp_resource_def,
                lp_mint_badge: Vault::with_bucket(lp_mint_badge),
                a_pool: Vault::with_bucket(a_tokens),
                b_pool: Vault::with_bucket(b_tokens),
                fee,
                lp_per_asset_ratio,
            }
            .instantiate();

            // Return the new Radiswap component, as well as the initial supply of LP tokens
            (radiswap, lp_tokens)
        }

        /// Adds liquidity to this pool and return the LP tokens representing pool shares
        /// along with any remainder.
        pub fn add_liquidity(&self, a_tokens: Bucket, b_tokens: Bucket) -> (Bucket, Bucket) {
            // Differentiate LP calculation based on whether pool is empty or not.
            let (supply_to_mint, remainder) = if self.lp_resource_def.total_supply() == 0.into() {
                // Set initial LP tokens based on previous LP per K ratio.
                let supply_to_mint =
                    self.lp_per_asset_ratio * a_tokens.amount() * b_tokens.amount();
                self.a_pool.put(a_tokens.take(a_tokens.amount()));
                self.b_pool.put(b_tokens);
                (supply_to_mint, a_tokens)
            } else {
                // The ratio of added liquidity in existing liquidty.
                let a_ratio = a_tokens.amount() / self.a_pool.amount();
                let b_ratio = b_tokens.amount() / self.b_pool.amount();

                let (actual_ratio, remainder) = if a_ratio <= b_ratio {
                    // We will claim all input token A's, and only the correct amount of token B
                    self.a_pool.put(a_tokens);
                    self.b_pool
                        .put(b_tokens.take(self.b_pool.amount() * a_ratio));
                    (a_ratio, b_tokens)
                } else {
                    // We will claim all input token B's, and only the correct amount of token A
                    self.b_pool.put(b_tokens);
                    self.a_pool
                        .put(a_tokens.take(self.a_pool.amount() * b_ratio));
                    (b_ratio, a_tokens)
                };
                (
                    self.lp_resource_def.total_supply() * actual_ratio,
                    remainder,
                )
            };

            // Mint LP tokens according to the share the provider is contributing
            let lp_tokens = self
                .lp_mint_badge
                .authorize(|auth| self.lp_resource_def.mint(supply_to_mint, auth));

            // Return the LP tokens along with any remainder
            (lp_tokens, remainder)
        }

        /// Removes liquidity from this pool.
        pub fn remove_liquidity(&self, lp_tokens: Bucket) -> (Bucket, Bucket) {
            assert!(
                self.lp_resource_def == lp_tokens.resource_def(),
                "Wrong token type passed in"
            );

            // Calculate the share based on the input LP tokens.
            let share = lp_tokens.amount() / self.lp_resource_def.total_supply();

            // Withdraw the correct amounts of tokens A and B from reserves
            let a_withdrawn = self.a_pool.take(self.a_pool.amount() * share);
            let b_withdrawn = self.b_pool.take(self.b_pool.amount() * share);

            // Burn the LP tokens received
            self.lp_mint_badge.authorize(|auth| {
                lp_tokens.burn(auth);
            });

            // Return the withdrawn tokens
            (a_withdrawn, b_withdrawn)
        }

        /// Swaps token A for B, or vice versa.
        pub fn swap(&mut self, input_tokens: Bucket) -> Bucket {
            // Calculate the swap fee
            let fee_amount = input_tokens.amount() * self.fee;

            let output_tokens = if input_tokens.resource_def() == self.a_pool.resource_def() {
                // Calculate how much of token B we will return
                let b_amount = self.b_pool.amount()
                    - self.a_pool.amount() * self.b_pool.amount()
                        / (input_tokens.amount() - fee_amount + self.a_pool.amount());

                // Put the input tokens into our pool
                self.a_pool.put(input_tokens);

                // Return the tokens owed
                self.b_pool.take(b_amount)
            } else {
                // Calculate how much of token A we will return
                let a_amount = self.a_pool.amount()
                    - self.a_pool.amount() * self.b_pool.amount()
                        / (input_tokens.amount() - fee_amount + self.b_pool.amount());

                // Put the input tokens into our pool
                self.b_pool.put(input_tokens);

                // Return the tokens owed
                self.a_pool.take(a_amount)
            };

            // Accrued fees change the raio
            self.lp_per_asset_ratio =
                self.lp_resource_def.total_supply() / (self.a_pool.amount() * self.b_pool.amount());

            output_tokens
        }

        /// Returns the resource definition addresses of the pair.
        pub fn get_pair(&self) -> (Address, Address) {
            (
                self.a_pool.resource_address(),
                self.b_pool.resource_address(),
            )
        }
    }
}

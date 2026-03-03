use anchor_lang::prelude::*;

declare_id!("");

#[program]
pub mod solanatiers {
    use super::*;

    pub fn initialize_creator(ctx: Context<InitializeCreator>, tier_prices: [u64; 4]) -> Result<()> {
        let creator = &mut ctx.accounts.creator_config;

        creator.authority = ctx.accounts.authority.key();
        creator.tier_prices = tier_prices;
        creator.bump = ctx.bumps.creator_config;

        Ok(())
    }
    
    pub fn subscribe(ctx: Context<Subscribe>, tier: u8, subscription_index: u64) -> Result<()> {

        require!(tier > 0, ErrorCode::InvalidTier);

        let creator_config = &ctx.accounts.creator_config;
        let price = creator_config.tier_prices[tier as usize];

        require!(price > 0, ErrorCode::InvalidTier);

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.subscriber.to_account_info(),
            to: ctx.accounts.creator.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            cpi_accounts,
        );

        anchor_lang::system_program::transfer(cpi_ctx, price)?;

        let subscription = &mut ctx.accounts.user_subscription;
        subscription.subscriber = ctx.accounts.subscriber.key();
        subscription.creator = creator_config.authority;
        subscription.tier = tier;
        subscription.index = subscription_index;
        subscription.bump = ctx.bumps.user_subscription;
        
        msg!("Suscripción #{} creada exitosamente para el Tier {}", subscription_index, tier);

        Ok(())
    }

    pub fn upgrade_tier(ctx: Context<UpgradeTier>, new_tier: u8, _subscription_index: u64) -> Result<()> {

        let subscription = &mut ctx.accounts.user_subscription;
        let creator_config = &ctx.accounts.creator_config;

        let current_tier = subscription.tier;

        require!(current_tier > 0, ErrorCode::NoActiveSubscription);
        require!(new_tier > current_tier, ErrorCode::InvalidTier);

        let current_price = creator_config.tier_prices[current_tier as usize];
        let new_price = creator_config.tier_prices[new_tier as usize];

        require!(new_price > 0, ErrorCode::InvalidTier);

        let difference = new_price - current_price;

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.subscriber.to_account_info(),
            to: ctx.accounts.creator.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            cpi_accounts,
        );

        anchor_lang::system_program::transfer(cpi_ctx, difference)?;

        subscription.tier = new_tier;

        Ok(())
    }

    pub fn cancel_subscription(_ctx: Context<CancelSubscription>, _subscription_index: u64) -> Result<()> {
        msg!("Suscripción cerrada exitosamente. Renta devuelta al suscriptor.");
        Ok(())
    }

    pub fn delete_creator_config(_ctx: Context<DeleteCreatorConfig>) -> Result<()> {
      
        Ok(())
    }

    pub fn check_access(ctx: Context<CheckAccess>, required_tier: u8) -> Result<bool> {
        let subscription = &ctx.accounts.user_subscription;

        msg!("--- Verificando Acceso ---");
        msg!("Suscriptor: {:?}", subscription.subscriber);
        msg!("Creador: {:?}", subscription.creator);
        msg!("Tier de la cuenta: {}", subscription.tier);
        msg!("Tier requerido: {}", required_tier);
        msg!("Índice de suscripción: {}", subscription.index);

        let has_access = subscription.tier >= required_tier;

        if has_access {
            msg!("Resultado: ACCESO CONCEDIDO ✅");
        } else {
            msg!("Resultado: ACCESO DENEGADO ❌");
        }
            
        Ok(has_access)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid tier selected")]
    InvalidTier,

    #[msg("No active subscription")]
    NoActiveSubscription
}

#[derive(Accounts)]
pub struct InitializeCreator<'info> {
    #[account(
        init,
        payer = authority,
        space = CreatorConfig::SPACE,
        seeds = [b"creator", authority.key().as_ref()],
        bump
    )]
    pub creator_config: Account<'info, CreatorConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(tier: u8, subscription_index: u64)]
pub struct Subscribe<'info> {
    #[account(
        mut,
        seeds = [b"creator", creator_config.authority.as_ref()],
        bump = creator_config.bump,
    )]
    pub creator_config: Account<'info, CreatorConfig>,

    #[account(
        init,
        payer = subscriber,
        space = UserSubscription::SPACE,
        seeds = [
            b"user",
            subscriber.key().as_ref(),
            creator_config.authority.as_ref(),
            &subscription_index.to_le_bytes()
        ],
        bump
    )]
    pub user_subscription: Account<'info, UserSubscription>,

    #[account(mut)]
    pub subscriber: Signer<'info>,

    #[account(
        mut,
        address = creator_config.authority
    )]
    pub creator: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(new_tier: u8, subscription_index: u64)]
pub struct UpgradeTier<'info> {
    #[account(
        mut,
        seeds = [b"creator", creator_config.authority.as_ref()],
        bump = creator_config.bump,
    )]
    pub creator_config: Account<'info, CreatorConfig>,

    #[account(
        mut,
        seeds = [
            b"user",
            subscriber.key().as_ref(),
            creator_config.authority.as_ref(),
            &subscription_index.to_le_bytes()
        ],
        bump,
        has_one = subscriber,
        has_one = creator
    )]
    pub user_subscription: Account<'info, UserSubscription>,

    #[account(mut)]
    pub subscriber: Signer<'info>,

    #[account(
        mut,
        address = creator_config.authority
    )]
    pub creator: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(subscription_index: u64)]
pub struct CancelSubscription<'info> {
    #[account(
        mut,
        seeds = [
            b"user",
            subscriber.key().as_ref(),
            creator_config.authority.as_ref(),
            &subscription_index.to_le_bytes()
        ],
        bump = user_subscription.bump,
        has_one = subscriber,
        has_one = creator,
        close = subscriber 
    )]
    pub user_subscription: Account<'info, UserSubscription>,

    #[account(mut)]
    pub subscriber: Signer<'info>,

    #[account(mut, address = creator_config.authority)]
    pub creator: SystemAccount<'info>,

    pub creator_config: Account<'info, CreatorConfig>,
}

#[derive(Accounts)]
pub struct DeleteCreatorConfig<'info> {
    #[account(
        mut,
        seeds = [b"creator", authority.key().as_ref()],
        bump = creator_config.bump,
        has_one = authority,
        close = authority
    )]
    pub creator_config: Account<'info, CreatorConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CheckAccess<'info> {
    pub user_subscription: Account<'info, UserSubscription>,
}


#[account]
pub struct CreatorConfig {
    pub authority: Pubkey,
    pub tier_prices: [u64; 4],
    pub bump: u8
}

impl CreatorConfig {
    pub const SPACE: usize = 8 + 32 + (8 * 4) + 1;
}

#[account]
pub struct UserSubscription {
    pub subscriber: Pubkey,
    pub creator: Pubkey,
    pub tier: u8,
    pub index: u64,
    pub bump: u8,
}

impl UserSubscription {
    pub const SPACE: usize = 8 + 32 + 32 + 1 + 8 + 1;
}

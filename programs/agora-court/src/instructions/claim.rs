use crate::error::AccountError;
use crate::{error::InputError, state::*};
use anchor_lang::prelude::*;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

//default settings -> see afterthought in vote.rs for future possibilities

//default: winning user gets full stake back, but no reward
//losers (parties and voters) do not get anything back
//winning voters divide up the pool evenly, hence the incentive to vote

//To-Do:
//TIES ARE NOT YET HANDLED
//Protocol needs way to withdraw provided tokens, impl through a user

//of course, for v2, need to prevent public visibility of current vote counts
//the claim queue only holds timestamps, not the status enum, which may delay claims unnecessarily.

pub fn claim(ctx: Context<Claim>, _dispute_id: u64) -> Result<()> {
    let dispute = &mut ctx.accounts.dispute;
    let dispute_rep_ata = &mut ctx.accounts.rep_vault;
    let voter_record = &mut ctx.accounts.voter_record;

    let _payer = &mut ctx.accounts.user;
    let _involved_with = voter_record.claim_queue.pop().unwrap().user_voted_for;

    let mut rep_amount_to_transfer = dispute.config.rep_cost;
    let mut pay_amount_to_transfer = dispute.config.pay_cost;

    match dispute.status {
        DisputeStatus::Concluded { winner: Some(x) } => {
            if x == _payer.key() {
                //refund arb_cost
            } else if dispute.users.contains(&Some(_payer.key())) {
                //losing party -= voter_record
                voter_record.currently_staked_pay -= pay_amount_to_transfer;
                voter_record.currently_staked_rep -= rep_amount_to_transfer;
                return Ok(());
            } else if x == _involved_with {
                //winning voter reward
                rep_amount_to_transfer = (((dispute.submitted_cases as u64 - 1) * dispute.config.rep_cost) + dispute.config.protocol_rep + (dispute.votes * dispute.config.voter_rep_cost)) / dispute.leader.votes;
                pay_amount_to_transfer = (((dispute.submitted_cases as u64 - 1) * dispute.config.pay_cost) + dispute.config.protocol_pay) / dispute.leader.votes;

                voter_record.currently_staked_pay -= dispute.config.voter_rep_cost;
            } else {
                //losing voter -= voter_record
                voter_record.currently_staked_rep -= dispute.config.voter_rep_cost;
                return Ok(());
            }
        },
        DisputeStatus::Concluded { winner: None } => {
            //refund arb_cost
        },
        _ => {
            return Ok(()); //impossible
        }
    }

    //refund arb_cost - two cases
    let court_key = ctx.accounts.court.key();
    let id_ne_bytes = u64::to_ne_bytes(_dispute_id);
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            "dispute".as_bytes(), 
            court_key.as_ref(), 
            id_ne_bytes.as_ref(),
            &[dispute.bump]
        ]
    ];

    if rep_amount_to_transfer > 0 {
        let user_ata = &mut ctx.accounts.user_rep_ata;

        if let Some(acc) = user_ata {
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: dispute_rep_ata.to_account_info(),
                    to: acc.to_account_info(),
                    authority: dispute.to_account_info()
                },
                signer_seeds
            );

            transfer(cpi_ctx, rep_amount_to_transfer)?;
            voter_record.currently_staked_rep -= rep_amount_to_transfer;
        } else {
            return err!(AccountError::ReputationAtaMissing);
        }
    }

    if pay_amount_to_transfer > 0 {
        let user_ata = &mut ctx.accounts.user_pay_ata;
        let vault_ata = &mut ctx.accounts.pay_vault;

        if let (Some(user_acc), Some(vault_acc), Some(mint), Some(mint_acc)) = (user_ata, vault_ata, &ctx.accounts.court.pay_mint, &ctx.accounts.pay_mint) {
            require!(mint_acc.key() == *mint, AccountError::ProtocolMintMismatch);

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: vault_acc.to_account_info(),
                    to: user_acc.to_account_info(),
                    authority: dispute.to_account_info()
                },
                signer_seeds
            );

            transfer(cpi_ctx, pay_amount_to_transfer)?;
            voter_record.currently_staked_pay -= pay_amount_to_transfer;
        } else {
            return err!(AccountError::PaymentAtaMissing);
        }
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(_dispute_id: u64)]
pub struct Claim<'info> {

    #[account(
        mut,
        seeds = ["record".as_bytes(), court.key().as_ref(), user.key().as_ref()],
        bump = voter_record.bump,
        constraint = voter_record.has_unclaimed_disputes()
                    @ InputError::UserHasNoUnclaimedDisputes,
        constraint = voter_record.claim_queue.peek().unwrap().dispute_id == _dispute_id
                    @ InputError::UserCannotClaimDispute,
    )]
    pub voter_record: Box<Account<'info, VoterRecord>>,

    #[account(
        mut,
        seeds = ["dispute".as_bytes(), court.key().as_ref(), u64::to_ne_bytes(_dispute_id).as_ref()],
        bump = dispute.bump,
        constraint = matches!(dispute.status, DisputeStatus::Concluded { .. })
                    @ InputError::DisputeNotClaimable,
   )]
    pub dispute: Box<Account<'info, Dispute>>,

    #[account(
        mut,
        associated_token::mint = rep_mint,
        associated_token::authority = dispute
    )]
    pub rep_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = pay_mint,
        associated_token::authority = dispute
    )]
    pub pay_vault: Option<Account<'info, TokenAccount>>,

    #[account(
        seeds = ["court".as_bytes(), court_authority.key().as_ref()],
        bump = court.bump,
    )]
    pub court: Box<Account<'info, Court>>,

    /// CHECK: The creator of the court should not need to sign here - it won't be the right court anyway if wrong address passed
    pub court_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = pay_mint,
        associated_token::authority = user
    )]
    pub user_pay_ata: Option<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = rep_mint,
        associated_token::authority = user,
    )]
    pub user_rep_ata: Option<Account<'info, TokenAccount>>,

    #[account(
        constraint = rep_mint.key() == court.rep_mint @ AccountError::ReputationMintMismatch
    )]
    pub rep_mint: Box<Account<'info, Mint>>,

    pub pay_mint: Option<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
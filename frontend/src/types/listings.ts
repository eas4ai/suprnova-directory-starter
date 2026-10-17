import type { SeoOverrides } from './seo'
export interface Category { id: number; slug: string; name: string }
export interface Revision { seo: SeoOverrides;
  id: number; title: string; summary: string; description: string; url: string;
  category_ids: number[]; media_id: string | null; media_alt: string;
  status: 'draft' | 'submitted' | 'approved' | 'rejected'; reason: string | null; media_url: string | null;
}
export interface OwnerListing {
  id: number; slug: string; version: number; archived: boolean; suspended: boolean;
  current: Revision; approved: Revision | null; moderation_status: string;
  payment_status: string; publication_status: string; next_action: string;
  purchase_id: string | null;
}
export interface Paged { page: number; per_page: number; total: number }
export interface PublicCard {
  id: number; slug: string; title: string; summary: string; url: string;
  media_url: string | null; media_alt: string; categories: Category[]; created_at: number;
}
export interface PublicDetail extends PublicCard { description_html: string }
export interface ListingInput { seo: SeoOverrides;
  version: number; title: string; summary: string; description: string; url: string;
  category_ids: number[]; media_id: string | null; media_alt: string;
}
